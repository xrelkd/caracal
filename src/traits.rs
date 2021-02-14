use mime::Mime;

use crate::{Error, MimeData, Mode};

pub trait Load {
    fn load(&self, mime: &Mime) -> Result<Vec<u8>, Error>;

    fn load_mime_data(&self) -> Result<MimeData, Error> {
        let mimes = &[
            mime::TEXT_PLAIN_UTF_8,
            mime::TEXT_PLAIN,
            mime::IMAGE_PNG,
            mime::IMAGE_JPEG,
            mime::IMAGE_GIF,
            mime::IMAGE_SVG,
            mime::IMAGE_BMP,
            mime::APPLICATION_OCTET_STREAM,
        ];

        for mime in mimes {
            match self.load(mime) {
                Ok(data) => return Ok(MimeData::new(mime.clone(), data)),
                Err(Error::MatchMime { .. }) => {
                    continue;
                }
                Err(Error::Empty) => return Err(Error::Empty),
                Err(err) => return Err(err),
            }
        }

        Err(Error::UnknownContentType)
    }

    fn is_empty(&self) -> bool { matches!(self.load_mime_data(), Err(Error::Empty)) }
}

pub trait Store {
    fn store(&self, mime: Mime, data: &[u8]) -> Result<(), Error>;

    fn store_mime_data(&self, mime_data: MimeData) -> Result<(), Error>;

    fn clear(&self) -> Result<(), Error>;
}

pub trait Wait {
    fn wait(&self) -> Result<Mode, Error>;
}

pub trait Subscribe {
    type Subscriber: Wait + Send;

    fn subscribe(&self) -> Result<Self::Subscriber, Error>;
}

pub trait LoadExt: Load {
    fn load_string(&self) -> Result<String, Error> {
        let data = self.load(&mime::TEXT_PLAIN_UTF_8)?;
        let data = String::from_utf8_lossy(&data);
        Ok(data.into())
    }
}

impl<C: Load + ?Sized> LoadExt for C {}

pub trait StoreExt: Store {
    fn store_string(&self, data: &str) -> Result<(), Error> {
        self.store(mime::TEXT_PLAIN_UTF_8, data.as_bytes())
    }
}

impl<C: Store + ?Sized> StoreExt for C {}

pub trait LoadWait: Load + Subscribe {
    fn load_wait(&self, mime: &Mime) -> Result<Vec<u8>, Error> {
        self.subscribe()?.wait()?;
        self.load(mime)
    }

    fn load_wait_mime_data(&self) -> Result<MimeData, Error> {
        self.subscribe()?.wait()?;
        self.load_mime_data()
    }
}

impl<C: Load + Subscribe + ?Sized> LoadWait for C {}
