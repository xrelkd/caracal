use std::time::{Duration, Instant};

use caracal::{
    ClipboardLoad, ClipboardLoadExt, ClipboardStore, ClipboardStoreExt, ClipboardSubscribe,
    ClipboardWait, Error,
};

pub trait ClipboardTester {
    type Clipboard: 'static
        + Clone
        + Sync
        + Send
        + ClipboardSubscribe
        + ClipboardLoad
        + ClipboardStore;

    fn new_clipboard(&self) -> Self::Clipboard;

    fn run(&self) -> Result<(), Error> {
        self.test_clean()?;

        for i in 0..20 {
            let data_size = 1 << i;
            println!("test with data size: {}", data_size);
            self.test_store_and_load(data_size)?;
            println!("passed, test with data size: {}", data_size);
        }

        self.test_subscribe()?;
        Ok(())
    }

    fn test_store_and_load(&self, len: usize) -> Result<(), Error> {
        let clipboard = self.new_clipboard();
        let mime = mime::TEXT_PLAIN_UTF_8;

        let original_data = vec!['A' as u8; len];
        clipboard.store(mime.clone(), &original_data)?;

        for _ in 0..5 {
            let loaded_data = clipboard.load(&mime)?;

            assert_eq!(loaded_data.len(), original_data.len());
            assert_eq!(loaded_data, original_data);
        }

        Ok(())
    }

    fn test_clean(&self) -> Result<(), Error> {
        let data = "This is a string";
        let clipboard = self.new_clipboard();

        clipboard.store(mime::TEXT_PLAIN_UTF_8, data.as_bytes())?;
        assert!(!clipboard.load(&mime::TEXT_PLAIN_UTF_8)?.is_empty());

        clipboard.clear()?;
        assert!(clipboard.is_empty());

        Ok(())
    }

    fn test_subscribe(&self) -> Result<(), Error> {
        let clipboard = self.new_clipboard();
        clipboard.clear()?;

        let observer = std::thread::spawn({
            let subscriber = clipboard.subscribe()?;
            let clipboard = clipboard.clone();
            move || -> Result<String, Error> {
                loop {
                    let _ = subscriber.wait();
                    match clipboard.load_string() {
                        Ok(data) => return Ok(data),
                        Err(Error::Empty) => continue,
                        Err(Error::MatchMime { .. }) => continue,
                        Err(err) => return Err(err),
                    }
                }
            }
        });

        let observer2 = std::thread::spawn({
            let subscriber = clipboard.subscribe()?;
            let clipboard = clipboard.clone();
            move || -> Result<String, Error> {
                loop {
                    let _ = subscriber.wait();
                    match clipboard.load_string() {
                        Ok(data) => return Ok(data),
                        Err(Error::Empty) => continue,
                        Err(Error::MatchMime { .. }) => continue,
                        Err(err) => return Err(err),
                    }
                }
            }
        });

        let observer3 = std::thread::spawn({
            let subscriber = clipboard.subscribe()?;
            move || -> Result<(), Error> {
                while let Ok(_) = subscriber.wait() {}
                Ok(())
            }
        });

        std::thread::sleep(Duration::from_millis(100));
        let input = format!("{:?}", Instant::now());
        clipboard.store_string(&input)?;

        let output = observer.join().unwrap()?;

        assert_eq!(input.len(), output.len());
        assert_eq!(input, output);

        let output2 = observer2.join().unwrap()?;
        assert_eq!(input.len(), output2.len());
        assert_eq!(input, output2);

        println!("drop clipboard");
        drop(clipboard);
        observer3.join().unwrap()?;

        Ok(())
    }
}
