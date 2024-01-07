use std::{io::Write, path::PathBuf};

use clap::{CommandFactory, Parser, Subcommand};
use snafu::ResultExt;
use tokio::runtime::Runtime;

use crate::{config::Config, error, error::Error, shadow, tui};

#[derive(Parser)]
#[command(
    name = caracal_base::TUI_PROGRAM_NAME,
    author,
    version,
    long_version = shadow::CLAP_LONG_VERSION,
    about,
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    commands: Option<Commands>,

    #[arg(long = "log-level", env = "CARACAL_TUI_LOG_LEVEL", help = "Specify a log level")]
    log_level: Option<tracing::Level>,

    #[arg(
        long = "config",
        short = 'C',
        env = "CARACAL_TUI_CONFIG_FILE_PATH",
        help = "Specify a configuration file"
    )]
    config_file: Option<PathBuf>,
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
    #[allow(clippy::too_many_lines)]
    pub fn run(self) -> Result<(), Error> {
        let Self { commands, log_level, config_file } = self;

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

        match commands {
            Some(Commands::Completions { .. } | Commands::DefaultConfig) => {
                unreachable!("these commands should be handled previously");
            }
            Some(Commands::Version) => {
                std::io::stdout()
                    .write_all(Self::command().render_long_version().as_bytes())
                    .expect("Failed to write to stdout");
                Ok(())
            }
            None => {
                Runtime::new().context(error::InitializeTokioRuntimeSnafu)?.block_on(async move {
                    tui::run(config.daemon.server_endpoint.clone(), config.daemon.access_token())
                        .await
                })
            }
        }
    }
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
