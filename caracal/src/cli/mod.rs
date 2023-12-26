mod standalone;

use std::{collections::HashMap, io::Write, path::PathBuf, time::Duration};

use caracal_base::profile::{minio::MinioAlias, ssh::SshConfig};
use caracal_cli::{
    profile,
    profile::{Profile, ProfileItem},
};
use caracal_engine::DownloaderFactory;
use clap::{CommandFactory, Parser, Subcommand};
use snafu::ResultExt;
use tokio::runtime::Runtime;

use crate::{
    config::Config,
    error::{self, Error},
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
}

impl Default for Cli {
    fn default() -> Self { Self::parse() }
}

impl Cli {
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
                None => {
                    let mut minio_aliases = HashMap::new();
                    let mut ssh_servers = HashMap::new();
                    for profile_file in config.profile_files() {
                        for profile_item in Profile::load(profile_file).await?.profiles {
                            match profile_item {
                                ProfileItem::Ssh(profile::Ssh {
                                    name,
                                    endpoint,
                                    user,
                                    identity_file,
                                }) => {
                                    let config = SshConfig {
                                        endpoint,
                                        user,
                                        identity_file: identity_file.to_string_lossy().to_string(),
                                    };
                                    drop(ssh_servers.insert(name, config));
                                }
                                ProfileItem::Minio(profile::Minio {
                                    name,
                                    endpoint_url,
                                    access_key,
                                    secret_key,
                                }) => {
                                    let alias = MinioAlias { endpoint_url, access_key, secret_key };
                                    drop(minio_aliases.insert(name, alias));
                                }
                            }
                        }
                    }

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
                _ => unreachable!(),
            }
        })
    }
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
