#![cfg(feature = "x11")]

use caracal::{Error, Mode, X11Clipboard};

mod common;

use self::common::ClipboardTester;

#[derive(Debug)]
pub struct X11ClipboardTester {
    mode: Mode,
}

impl X11ClipboardTester {
    pub fn new(mode: Mode) -> X11ClipboardTester { X11ClipboardTester { mode } }
}

impl ClipboardTester for X11ClipboardTester {
    type Clipboard = X11Clipboard;

    fn new_clipboard(&self) -> Self::Clipboard { X11Clipboard::new(None, self.mode).unwrap() }
}

#[test]
fn test_x11_clipboard() -> Result<(), Error> {
    let tester = X11ClipboardTester::new(Mode::Clipboard);
    tester.run()
}

#[test]
fn test_x11_selection() -> Result<(), Error> {
    let tester = X11ClipboardTester::new(Mode::Selection);
    tester.run()
}
