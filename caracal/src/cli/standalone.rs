use std::{collections::HashMap, future::Future, path::Path, pin::Pin, time::Duration};

use caracal_engine::{minio::MinioAlias, ssh::SshConfig, Factory, NewTask};
use futures::{FutureExt, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use sigfinn::{ExitStatus, LifecycleManager};
use snafu::ResultExt;

use crate::{error, error::Error};

const PROGRESS_STYLE_TEMPLATE: &str = "{spinner:.green} [{elapsed_precise}] [{msg}] \
                                       [{wide_bar:.cyan/blue}] {binary_bytes_per_sec} {percent}% \
                                       {bytes}/{total_bytes} ({eta})";

const CHUNK_SIZE: u64 = 100 * 1024;

enum Event {
    Shutdown,
    UpdateProgress,
}

pub async fn run<P>(
    urls: Vec<reqwest::Url>,
    output_directory: Option<P>,
    default_worker_number: u16,
    worker_number: Option<u16>,
    minio_aliases: HashMap<String, MinioAlias>,
    ssh_servers: HashMap<String, SshConfig>,
) -> Result<(), Error>
where
    P: AsRef<Path> + Send,
{
    if urls.is_empty() {
        return Err(Error::NoUrl);
    }

    let output_directory = if let Some(output_directory) = output_directory {
        output_directory.as_ref().to_path_buf()
    } else {
        std::env::current_dir().context(error::GetCurrentDirectorySnafu)?
    };

    if !output_directory.exists() {
        return Err(Error::OutputDirectoryNotExists { output_directory });
    }
    if output_directory.is_file() {
        return Err(Error::OutputDirectoryPathIsFile { output_directory });
    }

    let factory =
        Factory::new(u64::from(default_worker_number), CHUNK_SIZE, minio_aliases, ssh_servers);
    let multi_progress = MultiProgress::new();
    let sty = ProgressStyle::with_template(PROGRESS_STYLE_TEMPLATE)
        .expect("valid template")
        .progress_chars("##-");

    let lifecycle_manager = LifecycleManager::<Error>::new();

    for url in urls {
        let task = NewTask {
            url: url.clone(),
            directory_path: output_directory.clone(),
            filename: None,
            worker_number: worker_number.map(u64::from),
        };

        let progress_bar = multi_progress.add(ProgressBar::new(0));
        progress_bar.set_style(sty.clone());

        let _handle = lifecycle_manager
            .spawn(format!("{url}"), create_task_future(task, factory.clone(), progress_bar));
    }

    lifecycle_manager.serve().await.context(error::LifecycleManagerSnafu)?
}

fn create_task_future(
    task: NewTask,
    factory: Factory,
    progress_bar: ProgressBar,
) -> impl FnOnce(sigfinn::Shutdown) -> Pin<Box<dyn Future<Output = ExitStatus<Error>> + Send>> {
    move |shutdown| {
        async move {
            let url = task.url.clone();
            let mut downloader = match factory.create_new_task(task).await {
                Ok(d) => d,
                Err(error) => {
                    eprintln!("{error}");
                    return sigfinn::ExitStatus::Failure(Error::Downloader {
                        url: Box::new(url),
                        error,
                    });
                }
            };

            if let Err(error) = downloader.start().await {
                tracing::error!("{error}");
                return sigfinn::ExitStatus::Failure(Error::Downloader {
                    url: Box::new(url),
                    error,
                });
            }

            let mut shutdown = shutdown.into_stream();

            loop {
                let event = tokio::select! {
                    _ = shutdown.next() => Event::Shutdown,
                    () = tokio::time::sleep(Duration::from_millis(200)) => Event::UpdateProgress,
                };
                match event {
                    Event::Shutdown => {
                        if let Err(err) = downloader.pause().await {
                            eprintln!("{err}");
                        }
                        break;
                    }
                    Event::UpdateProgress => {
                        if let Some(progress) = downloader.progress().await {
                            progress_bar.set_position(progress.total_received());
                            progress_bar.set_length(progress.content_length());
                            progress_bar.set_message(format!(
                                "{}/{} {}",
                                progress.completed_chunk_count(),
                                progress.total_chunk_count(),
                                progress.filename()
                            ));
                            if progress.is_completed() {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
            }

            match downloader.join().await {
                Ok(Some((_transfer_status, progress))) => {
                    progress_bar.finish_with_message(format!(
                        "{}/{} {}",
                        progress.completed_chunk_count(),
                        progress.total_chunk_count(),
                        progress.filename()
                    ));
                    sigfinn::ExitStatus::Success
                }
                Ok(_) => sigfinn::ExitStatus::Success,
                Err(error) => {
                    tracing::error!("{error}");
                    sigfinn::ExitStatus::Failure(Error::Downloader { url: Box::new(url), error })
                }
            }
        }
        .boxed()
    }
}
