#![cfg(feature = "x11")]
use snafu::ErrorCompat;

use caracal::{ClipboardLoad, Error, Mode};

use caracal::X11Clipboard;

fn main() -> Result<(), Error> {
    let clipboard = X11Clipboard::new(None, Mode::Clipboard)?;
    match clipboard.load_mime_data() {
        Ok(mime_data) => {
            let (mime, data) = mime_data.destruct();
            println!("content type: {}", mime.essence_str());
            println!("size: {}", data.len());
            if mime == mime::TEXT_PLAIN_UTF_8 {
                println!("data: \"{}\"", String::from_utf8_lossy(&data));
            }
            Ok(())
        }
        Err(Error::Empty) => {
            eprintln!("error: clipboard is empty");
            Err(Error::Empty)
        }
        Err(err) => {
            eprintln!("{}", err);
            if let Some(backtrace) = ErrorCompat::backtrace(&err) {
                eprintln!("{}", backtrace);
            }
            Err(err)
        }
    }
}
