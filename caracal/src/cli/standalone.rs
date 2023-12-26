use std::{future::Future, path::Path, pin::Pin, sync::Arc, time::Duration};

use caracal_engine::{DownloaderFactory, NewTask};
use futures::{FutureExt, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use sigfinn::{ExitStatus, LifecycleManager};
use snafu::ResultExt;

use crate::{error, error::Error};

const PROGRESS_STYLE_TEMPLATE: &str = "{spinner:.green} [{elapsed_precise}] [{msg}] \
                                       [{wide_bar:.cyan/blue}] {binary_bytes_per_sec} {percent}% \
                                       {bytes}/{total_bytes} ({eta})";

enum Event {
    Shutdown,
    UpdateProgress,
}

pub async fn run<P>(
    uris: Vec<http::Uri>,
    output_directory: Option<P>,
    worker_number: Option<u16>,
    connection_timeout: Option<Duration>,
    downloader_factory: DownloaderFactory,
) -> Result<(), Error>
where
    P: AsRef<Path> + Send,
{
    if uris.is_empty() {
        return Err(Error::NoUri);
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

    let downloader_factory = Arc::new(downloader_factory);
    let multi_progress = MultiProgress::new();
    let sty = ProgressStyle::with_template(PROGRESS_STYLE_TEMPLATE)
        .expect("valid template")
        .progress_chars("##-");

    let lifecycle_manager = LifecycleManager::<Error>::new();

    for (idx, uri) in uris.into_iter().enumerate() {
        let task = NewTask {
            uri,
            directory_path: output_directory.clone(),
            filename: None,
            worker_number: worker_number.map(u64::from),
            connection_timeout,
        };

        let progress_bar = multi_progress.add(ProgressBar::new(0));
        progress_bar.set_style(sty.clone());

        let _handle = lifecycle_manager.spawn(
            format!("Downloader {idx}"),
            create_task_future(task, downloader_factory.clone(), progress_bar),
        );
    }

    lifecycle_manager.serve().await.context(error::LifecycleManagerSnafu)?
}

fn create_task_future(
    task: NewTask,
    factory: Arc<DownloaderFactory>,
    progress_bar: ProgressBar,
) -> impl FnOnce(sigfinn::Shutdown) -> Pin<Box<dyn Future<Output = ExitStatus<Error>> + Send>> {
    move |shutdown| {
        async move {
            let mut downloader = match factory.create_new_task(&task).await {
                Ok(d) => d,
                Err(error) => {
                    eprintln!("{error}");
                    return sigfinn::ExitStatus::Error(Error::Downloader {
                        uri: Box::new(task.uri),
                        error,
                    });
                }
            };

            if let Err(error) = downloader.start().await {
                tracing::error!("{error}");
                return sigfinn::ExitStatus::Error(Error::Downloader {
                    uri: Box::new(task.uri),
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
                    progress_bar.set_position(progress.total_received());
                    progress_bar.set_length(progress.content_length());
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
                    sigfinn::ExitStatus::Error(Error::Downloader { uri: Box::new(task.uri), error })
                }
            }
        }
        .boxed()
    }
}
