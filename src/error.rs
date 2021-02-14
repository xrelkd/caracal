use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum Error {
    #[snafu(display("Failed to parse MIME, value: {}, error: {}", value.to_string(), source))]
    ParseMime { source: mime::FromStrError, value: String },

    #[snafu(display("Target type is not matched, expected content type: {}", expected_mime.essence_str()))]
    MatchMime { expected_mime: mime::Mime },

    #[snafu(display("Unknown content type"))]
    UnknownContentType,

    #[snafu(display("Clipboard is empty"))]
    Empty,

    #[snafu(display("Clipboard monitor is closed"))]
    ClipboardMonitorClosed,

    #[snafu(display("Primitive was poisoned"))]
    PrimitivePoisoned,

    #[snafu(display("Notifier is closed"))]
    NotifierClosed,

    #[snafu(display("X11 worker thread is failed"))]
    X11WorkerThreadFailed,

    #[snafu(display("Xfixes is not present"))]
    XfixesNotPresent,

    #[cfg(feature = "x11")]
    #[snafu(display("Reply error: {}", source))]
    Reply { source: x11rb::errors::ReplyError, backtrace: snafu::Backtrace },

    #[cfg(feature = "x11")]
    #[snafu(display("Could not create X11 connection, error: {}", source))]
    Connect { source: x11rb::errors::ConnectError, backtrace: snafu::Backtrace },

    #[cfg(feature = "x11")]
    #[snafu(display("Could not generate X11 identifier, error: {}", source))]
    GenerateX11Identifier { source: x11rb::errors::ReplyOrIdError, backtrace: snafu::Backtrace },

    #[cfg(feature = "x11")]
    #[snafu(display("Could not create new window, error: {}", source))]
    CreateWindow { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[cfg(feature = "x11")]
    #[snafu(display("Could not flush connection, error: {}", source))]
    FlushConnection { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[cfg(feature = "x11")]
    #[snafu(display("Could not send event, error: {}", source))]
    SendEvent { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[cfg(feature = "x11")]
    #[snafu(display("Could not get selection owner, error: {}", source))]
    GetSelectionOwner { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[cfg(feature = "x11")]
    #[snafu(display("Could not claim selection owner, error: {}", source))]
    ClaimSelectionOwner { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[cfg(feature = "x11")]
    #[snafu(display("Error occurs while releasing selection owner, error: {}", source))]
    ReleaseSelectionOwner { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[cfg(feature = "x11")]
    #[snafu(display("Selection owner is not matched"))]
    MatchSelectionOwner,

    #[cfg(feature = "x11")]
    #[snafu(display("Could not change property, error: {}", source))]
    ChangeProperty { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[cfg(feature = "x11")]
    #[snafu(display("Could not change window attributes, error: {}", source))]
    ChangeWindowAttributes { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[cfg(feature = "x11")]
    #[snafu(display("Could not get atom identifier by name {}, error: {}", atom_name, source))]
    GetAtomIdentifierByName {
        atom_name: String,
        source: x11rb::errors::ConnectionError,
        backtrace: snafu::Backtrace,
    },

    #[cfg(feature = "x11")]
    #[snafu(display("Could not get atom name, error: {}", source))]
    GetAtomName { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[cfg(feature = "x11")]
    #[snafu(display("Could not get property reply, error: {}", source))]
    GetPropertyReply { source: x11rb::errors::ReplyError, backtrace: snafu::Backtrace },

    #[cfg(feature = "x11")]
    #[snafu(display("Could not get property, error: {}", source))]
    GetProperty { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[cfg(feature = "x11")]
    #[snafu(display("Could not convert selection, error: {}", source))]
    ConvertSelection { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[cfg(feature = "x11")]
    #[snafu(display("Could not delete property, error: {}", source))]
    DeleteProperty { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[cfg(feature = "x11")]
    #[snafu(display("Error occurs while waiting for event, error: {}", source))]
    WaitForEvent { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[cfg(feature = "x11")]
    #[snafu(display("Error occurs while polling for event, error: {}", source))]
    PollForEvent { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[cfg(feature = "x11")]
    #[snafu(display("Could not query extension: {}, error: {}", extension_name, source))]
    QueryExtension {
        extension_name: String,
        source: x11rb::errors::ConnectionError,
        backtrace: snafu::Backtrace,
    },

    #[cfg(feature = "x11")]
    #[snafu(display("Could not query Xfixes version, error: {}", source))]
    QueryXfixesVersion { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[cfg(feature = "x11")]
    #[snafu(display("Could not select Xfixes selection input, error: {}", source))]
    SelectXfixesSelectionInput {
        source: x11rb::errors::ConnectionError,
        backtrace: snafu::Backtrace,
    },
}
