use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Failed to enable raw mode, error: {source}"))]
    EnableRawMode { source: std::io::Error },

    #[snafu(display("Failed to disable raw mode, error: {source}"))]
    DisableRawMode { source: std::io::Error },

    #[snafu(display("Failed to show cursor, error: {source}"))]
    ShowCursor { source: std::io::Error },

    #[snafu(display("Failed to create terminal, error: {source}"))]
    CreateTerminal { source: std::io::Error },

    #[snafu(display("Unable to switch to main screen, error: {source}"))]
    EnterMainScreen { source: std::io::Error },

    #[snafu(display("Unable to enter alternate screen, error: {source}"))]
    EnterAlternateScreen { source: std::io::Error },

    #[snafu(display("Could not render to the terminal, error: {source}"))]
    RenderTerminal { source: std::io::Error },
}
