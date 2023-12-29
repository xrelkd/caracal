mod standalone;
mod ui;

use std::{io::Write, path::PathBuf, time::Duration};

use caracal_base::{model, model::Priority};
use caracal_engine::DownloaderFactory;
use caracal_grpc_client as grpc;
use caracal_grpc_client::Task as _;
use clap::{CommandFactory, Parser, Subcommand};
use snafu::ResultExt;
use time::OffsetDateTime;
use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::{
    config::{self, Config},
    error,
    error::Error,
    shadow,
};

const MINIMUM_CHUNK_SIZE: u64 = 100 * 1024;

#[derive(Parser)]
#[command(
    name = caracal_base::CLI_PROGRAM_NAME,
    author,
    version,
    long_version = shadow::CLAP_LONG_VERSION,
    about,
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    commands: Option<Commands>,

    #[arg(long = "log-level", env = "CARACAL_LOG_LEVEL", help = "Specify a log level")]
    log_level: Option<tracing::Level>,

    #[arg(
        long = "config",
        short = 'C',
        env = "CARACAL_CONFIG_FILE_PATH",
        help = "Specify a configuration file"
    )]
    config_file: Option<PathBuf>,

    #[arg(
        long = "output-directory",
        short = 'D',
        help = "The directory to store the downloaded files"
    )]
    output_directory: Option<PathBuf>,

    #[arg(
        long = "num-connections",
        short = 'n',
        help = "Specify an alternative number of connections"
    )]
    concurrent_connections: Option<u16>,

    #[arg(long = "timeout", short = 'T', help = "Set the network timeout to seconds")]
    connection_timeout: Option<u64>,

    uris: Vec<http::Uri>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[clap(about = "Print version information")]
    Version,

    #[clap(about = "Output shell completion code for the specified shell (bash, zsh, fish)")]
    Completions { shell: clap_complete::Shell },

    #[clap(about = "Output default configuration")]
    DefaultConfig,

    #[clap(about = "Add URI to daemon")]
    AddUri {
        #[arg(long = "pause", help = "Add new URI but not start immediately")]
        pause: bool,

        #[arg(
            long = "priority",
            default_value = "normal",
            help = "Set the priority, available values: \"lowest\", \"low\", \"normal\", \
                    \"high\", \"highest\""
        )]
        priority: Priority,

        #[arg(
            long = "output-directory",
            short = 'D',
            help = "The directory to store the downloaded files"
        )]
        output_directory: Option<PathBuf>,

        #[arg(
            long = "num-connections",
            short = 'n',
            help = "Specify an alternative number of connections"
        )]
        concurrent_connections: Option<u16>,

        #[arg(long = "timeout", short = 'T', help = "Set the network timeout to seconds")]
        connection_timeout: Option<u64>,

        uris: Vec<http::Uri>,
    },

    #[clap(about = "Get a status of a specified task")]
    GetTaskStatus {
        #[arg(help = "Task ID")]
        id: Uuid,
    },

    #[clap(about = "Get status of all tasks")]
    GetAllTaskStatuses,

    #[clap(about = "Pause a task")]
    PauseTask {
        #[arg(help = "Task ID")]
        id: Uuid,
    },

    #[clap(about = "Pause all tasks")]
    PauseAllTasks,

    #[clap(about = "Resume a task")]
    ResumeTask {
        #[arg(help = "Task ID")]
        id: Uuid,
    },

    #[clap(about = "Resume all tasks")]
    ResumeAllTasks,

    #[clap(about = "Remove a task")]
    RemoveTask {
        #[arg(help = "Task ID")]
        id: Uuid,
    },
}

impl Default for Cli {
    fn default() -> Self { Self::parse() }
}

impl Cli {
    #[allow(clippy::too_many_lines)]
    pub fn run(self) -> Result<(), Error> {
        let Self {
            commands,
            log_level,
            config_file,
            output_directory,
            concurrent_connections,
            connection_timeout,
            uris,
        } = self;

        match commands {
            Some(Commands::Version) => {
                std::io::stdout()
                    .write_all(Self::command().render_long_version().as_bytes())
                    .expect("Failed to write to stdout");
                return Ok(());
            }
            Some(Commands::Completions { shell }) => {
                let mut app = Self::command();
                let bin_name = app.get_name().to_string();
                clap_complete::generate(shell, &mut app, bin_name, &mut std::io::stdout());
                return Ok(());
            }
            Some(Commands::DefaultConfig) => {
                let config_text =
                    toml::to_string_pretty(&Config::default()).expect("Config is serializable");
                std::io::stdout()
                    .write_all(config_text.as_bytes())
                    .expect("Failed to write to stdout");
                return Ok(());
            }
            _ => {}
        }

        let mut config = Config::load_or_default(config_file.unwrap_or_else(Config::default_path));
        if let Some(log_level) = log_level {
            config.log.level = log_level;
        }

        config.log.registry();

        Runtime::new().context(error::InitializeTokioRuntimeSnafu)?.block_on(async move {
            match commands {
                Some(
                    Commands::Version | Commands::Completions { .. } | Commands::DefaultConfig,
                ) => {
                    unreachable!("these commands should be handled previously");
                }
                Some(Commands::AddUri {
                    pause,
                    priority,
                    output_directory,
                    connection_timeout,
                    concurrent_connections,
                    uris,
                }) => {
                    let client = create_grpc_client(&config).await?;
                    let start_immediately = !pause;
                    let output_directory = output_directory.clone().unwrap_or(
                        std::env::current_dir().context(error::GetCurrentDirectorySnafu)?,
                    );
                    for uri in uris {
                        let create_task = model::CreateTask {
                            uri,
                            filename: None,
                            output_directory: output_directory.clone(),
                            connection_timeout: connection_timeout.map(Duration::from_secs),
                            concurrent_number: concurrent_connections.map(u64::from),
                            priority,
                            creation_timestamp: OffsetDateTime::now_utc(),
                        };
                        let task_id = client.add_uri(create_task, start_immediately).await?;
                        println!("{task_id}");
                    }
                    drop(client);
                    Ok(())
                }
                Some(Commands::GetTaskStatus { id }) => {
                    let client = create_grpc_client(&config).await?;
                    let task_status = client.get_task_status(id).await?;
                    println!("{}", ui::render_task_statuses_table(&[task_status]));
                    drop(client);
                    Ok(())
                }
                Some(Commands::GetAllTaskStatuses) => {
                    let client = create_grpc_client(&config).await?;
                    let task_statuses = client.get_all_task_statuses().await?;
                    println!("{}", ui::render_task_statuses_table(&task_statuses));
                    drop(client);
                    Ok(())
                }
                Some(Commands::PauseTask { id }) => {
                    let client = create_grpc_client(&config).await?;
                    let _ = client.pause(id).await?;
                    drop(client);
                    Ok(())
                }
                Some(Commands::PauseAllTasks) => {
                    let client = create_grpc_client(&config).await?;
                    let task_ids = client.pause_all().await?;
                    for task_id in task_ids {
                        println!("{task_id} is paused");
                    }
                    drop(client);
                    Ok(())
                }
                Some(Commands::ResumeTask { id }) => {
                    let client = create_grpc_client(&config).await?;
                    let _ = client.resume(id).await?;
                    drop(client);
                    Ok(())
                }
                Some(Commands::ResumeAllTasks) => {
                    let client = create_grpc_client(&config).await?;
                    let task_ids = client.resume_all().await?;
                    for task_id in task_ids {
                        println!("{task_id} is resumed");
                    }
                    drop(client);
                    Ok(())
                }
                Some(Commands::RemoveTask { id }) => {
                    let client = create_grpc_client(&config).await?;
                    let _ = client.remove(id).await?;
                    drop(client);
                    Ok(())
                }
                None => {
                    let config::Profiles { ssh_servers, minio_aliases } =
                        config.load_profiles().await.context(error::ConfigSnafu)?;
                    let downloader_factory = DownloaderFactory::builder()
                        .http_user_agent(config.downloader.http.user_agent)
                        .default_worker_number(u64::from(
                            config.downloader.http.concurrent_connections,
                        ))
                        .minimum_chunk_size(MINIMUM_CHUNK_SIZE)
                        .ssh_servers(ssh_servers)
                        .minio_aliases(minio_aliases)
                        .build()
                        .context(error::InitializeDownloaderSnafu)?;

                    standalone::run(
                        uris,
                        output_directory,
                        concurrent_connections,
                        connection_timeout.map(Duration::from_secs),
                        downloader_factory,
                    )
                    .await
                }
            }
        })
    }
}

async fn create_grpc_client(config: &Config) -> Result<grpc::Client, Error> {
    let server_endpoint = config.daemon.server_endpoint.clone();
    let access_token = config.daemon.access_token();
    Ok(grpc::Client::new(server_endpoint, access_token).await?)
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::{Cli, Commands};

    #[test]
    fn test_command_simple() {
        match Cli::parse_from(["program_name", "version"]).commands {
            Some(Commands::Version { .. }) => (),
            _ => panic!(),
        }
    }
}
