mod standalone;
mod ui;

use std::{io::Write, path::PathBuf, time::Duration};

use caracal_base::{model, model::Priority};
use caracal_engine::{DownloaderFactory, MINIMUM_CHUNK_SIZE};
use caracal_grpc_client as grpc;
use caracal_grpc_client::Task as _;
use clap::{CommandFactory, Parser, Subcommand};
use grpc::System;
use snafu::ResultExt;
use time::OffsetDateTime;
use tokio::runtime::Runtime;

use crate::{
    config::{self, Config},
    error,
    error::Error,
    shadow,
};

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

    #[arg(long = "timeout", short = 'T', help = "Set the network timeout in second")]
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

        #[arg(long = "timeout", short = 'T', help = "Set the network timeout in second")]
        connection_timeout: Option<u64>,

        uris: Vec<http::Uri>,
    },

    #[clap(about = "Get status of all tasks")]
    Status {
        #[arg(help = "Task ID")]
        id: Option<u64>,
    },

    #[clap(about = "Pause tasks")]
    Pause {
        #[arg(long = "all", short = 'a', help = "Pause all tasks")]
        all: bool,

        #[arg(help = "Task ID")]
        ids: Vec<u64>,
    },

    #[clap(about = "Resume tasks")]
    Resume {
        #[arg(long = "all", short = 'a', help = "Resume all tasks")]
        all: bool,

        #[arg(help = "Task ID")]
        ids: Vec<u64>,
    },

    #[clap(about = "Remove tasks")]
    Remove {
        #[arg(help = "Task ID")]
        ids: Vec<u64>,
    },

    #[clap(about = "Increase concurrent number of tasks")]
    IncreaseConcurrentNumber {
        #[arg(help = "Task ID")]
        ids: Vec<u64>,
    },

    #[clap(about = "Decrease concurrent number of tasks")]
    DecreaseConcurrentNumber {
        #[arg(help = "Task ID")]
        ids: Vec<u64>,
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
                Some(Commands::Completions { .. } | Commands::DefaultConfig) => {
                    unreachable!("these commands should be handled previously");
                }
                Some(Commands::Version) => {
                    std::io::stdout()
                        .write_all(Self::command().render_long_version().as_bytes())
                        .expect("Failed to write to stdout");

                    let maybe_client = create_grpc_client(&config).await;
                    if let Ok(ref client) = maybe_client {
                        let client_version =
                            Self::command().get_version().unwrap_or_default().to_string();
                        let server_version = client.get_version().await.map_or_else(
                            |_err| "unknown".to_string(),
                            |version| version.to_string(),
                        );
                        let info = format!(
                            "Client:\n\tVersion: {client_version}\n\nServer:\n\tVersion: \
                             {server_version}\n",
                        );
                        std::io::stdout()
                            .write_all(info.as_bytes())
                            .expect("Failed to write to stdout");
                    }
                    drop(maybe_client);
                    Ok(())
                }
                Some(Commands::AddUri {
                    pause,
                    priority,
                    output_directory,
                    connection_timeout,
                    concurrent_connections,
                    uris,
                }) => {
                    let output_directory = if let Some(path) = output_directory {
                        tokio::fs::canonicalize(path).await.ok()
                    } else {
                        None
                    };
                    let start_immediately = !pause;
                    let client = create_grpc_client(&config).await?;
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
                Some(Commands::Status { id }) => {
                    let client = create_grpc_client(&config).await?;
                    let mut task_statuses = if let Some(id) = id {
                        vec![client.get_task_status(id).await?]
                    } else {
                        client.get_all_task_statuses().await?
                    };
                    task_statuses.sort_unstable_by_key(|status| status.id);
                    println!("{table}", table = ui::render_task_statuses_table(&task_statuses));
                    drop(client);
                    Ok(())
                }
                Some(Commands::Pause { ids, all }) => {
                    let client = create_grpc_client(&config).await?;
                    let task_ids = if all {
                        client.pause_all().await?
                    } else {
                        for &id in &ids {
                            let _ = client.pause(id).await?;
                        }
                        ids
                    };
                    for task_id in task_ids {
                        println!("{task_id} is paused");
                    }
                    drop(client);
                    Ok(())
                }
                Some(Commands::Resume { ids, all }) => {
                    let client = create_grpc_client(&config).await?;
                    let task_ids = if all {
                        client.resume_all().await?
                    } else {
                        for &id in &ids {
                            let _ = client.resume(id).await?;
                        }
                        ids
                    };
                    for id in task_ids {
                        println!("{id} is resumed");
                    }
                    drop(client);
                    Ok(())
                }
                Some(Commands::Remove { ids }) => {
                    let client = create_grpc_client(&config).await?;
                    for &id in &ids {
                        let _ = client.remove(id).await?;
                    }
                    for id in ids {
                        println!("{id} is removed");
                    }
                    drop(client);
                    Ok(())
                }
                Some(Commands::IncreaseConcurrentNumber { ids }) => {
                    let client = create_grpc_client(&config).await?;
                    for &id in &ids {
                        let _ = client.increase_concurrent_number(id).await?;
                    }
                    drop(client);
                    Ok(())
                }
                Some(Commands::DecreaseConcurrentNumber { ids }) => {
                    let client = create_grpc_client(&config).await?;
                    for &id in &ids {
                        let _ = client.decrease_concurrent_number(id).await?;
                    }

                    drop(client);
                    Ok(())
                }
                None => {
                    let config::Profiles { ssh_servers, minio_aliases } =
                        config.load_profiles().await.context(error::ConfigSnafu)?;
                    let downloader_factory = DownloaderFactory::builder()
                        .context(error::BuildDownloaderFactorySnafu)?
                        .http_user_agent(config.downloader.http.user_agent)
                        .default_concurrent_number(u64::from(
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
        if let Some(Commands::Version { .. }) =
            Cli::parse_from(["program_name", "version"]).commands
        {
            // everything is good.
        } else {
            panic!();
        }
    }
}
