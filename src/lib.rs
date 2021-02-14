mod error;
mod mime_data;
mod mock;
mod pubsub;
pub mod traits;

#[cfg(feature = "x11")]
mod x11;
#[cfg(feature = "x11")]
pub use self::x11::Clipboard as X11Clipboard;

pub use self::pubsub::Subscriber;

pub use self::{
    error::Error,
    mime_data::MimeData,
    mock::Clipboard as MockClipboard,
    traits::{
        Load as ClipboardLoad, LoadExt as ClipboardLoadExt, LoadWait as ClipboardLoadWait,
        Store as ClipboardStore, StoreExt as ClipboardStoreExt, Subscribe as ClipboardSubscribe,
        Wait as ClipboardWait,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mode {
    Clipboard,
    Selection,
}

impl Default for Mode {
    fn default() -> Mode { Mode::Clipboard }
}
