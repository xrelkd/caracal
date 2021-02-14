use std::sync::{Arc, RwLock};

use crate::{
    pubsub::{self, Publisher, Subscriber},
    ClipboardLoad, ClipboardStore, ClipboardSubscribe, Error, MimeData, Mode,
};

#[derive(Clone, Debug)]
pub struct Clipboard {
    data: Arc<RwLock<Option<MimeData>>>,
    publisher: Arc<Publisher>,
    subscriber: Subscriber,
}

impl Default for Clipboard {
    fn default() -> Clipboard {
        let (publisher, subscriber) = pubsub::new(Mode::Clipboard);
        let data = Default::default();
        Clipboard { publisher: Arc::new(publisher), subscriber, data }
    }
}

impl Clipboard {
    #[inline]
    pub fn new() -> Clipboard { Clipboard::default() }

    #[inline]
    pub fn with_content(mime: mime::Mime, content: &[u8]) -> Clipboard {
        let mime_data = MimeData::new(mime, content.into());
        let data = Arc::new(RwLock::new(Some(mime_data)));
        let (publisher, subscriber) = pubsub::new(Mode::Clipboard);
        Clipboard { data, publisher: Arc::new(publisher), subscriber }
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) { self.publisher.close(); }
}

impl ClipboardSubscribe for Clipboard {
    type Subscriber = Subscriber;

    fn subscribe(&self) -> Result<Subscriber, Error> { Ok(self.subscriber.clone()) }
}

impl ClipboardLoad for Clipboard {
    fn load(&self, _: &mime::Mime) -> Result<Vec<u8>, Error> {
        let data = self.data.read().unwrap();
        match *data {
            Some(ref data) => Ok(data.clone_bytes()),
            None => Err(Error::Empty),
        }
    }
}

impl ClipboardStore for Clipboard {
    fn store(&self, mime: mime::Mime, data: &[u8]) -> Result<(), Error> {
        let mime_data = MimeData::new(mime, data.into());
        self.store_mime_data(mime_data)
    }

    #[inline]
    fn store_mime_data(&self, mime_data: MimeData) -> Result<(), Error> {
        match self.data.write() {
            Ok(mut data) => {
                *data = Some(mime_data);
                self.publisher.notify_all();
                Ok(())
            }
            Err(_err) => Err(Error::PrimitivePoisoned),
        }
    }

    fn clear(&self) -> Result<(), Error> {
        match self.data.write() {
            Ok(mut data) => {
                *data = None;
                Ok(())
            }
            Err(_err) => Err(Error::PrimitivePoisoned),
        }
    }
}
