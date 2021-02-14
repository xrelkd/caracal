#![cfg(feature = "x11")]
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use snafu::ErrorCompat;

use caracal::{ClipboardStore, Mode, X11Clipboard};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let clipboard = X11Clipboard::new(None, Mode::Clipboard)?;
    let data = format!("{:?}", Instant::now());
    match clipboard.store(mime::TEXT_PLAIN_UTF_8, data.as_bytes()) {
        Ok(_) => {
            println!("Press Ctrl-C to stop providing text: {}", data);
            println!("You can try to paste text into other window");
            let term = Arc::new(AtomicBool::new(false));
            signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;
            signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term))?;

            while !term.load(Ordering::Relaxed) {
                let _ = std::thread::sleep(Duration::from_millis(100));
            }

            println!("Exit");
            Ok(())
        }
        Err(err) => {
            eprintln!("{}", err);
            if let Some(backtrace) = ErrorCompat::backtrace(&err) {
                eprintln!("{}", backtrace);
            }
            Err(err)?
        }
    }
}
