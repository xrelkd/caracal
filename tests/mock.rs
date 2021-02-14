use caracal::{Error, MockClipboard};

mod common;

use self::common::ClipboardTester;

#[derive(Debug)]
pub struct MockClipboardTester;

impl MockClipboardTester {
    pub fn new() -> MockClipboardTester { MockClipboardTester }
}

impl ClipboardTester for MockClipboardTester {
    type Clipboard = MockClipboard;

    fn new_clipboard(&self) -> Self::Clipboard { MockClipboard::new() }
}

#[test]
fn test_mock() -> Result<(), Error> {
    let tester = MockClipboardTester::new();
    tester.run()
}
