use bytes::BytesMut;
use snafu::ResultExt;
use tokio::io::AsyncReadExt;

use crate::{error, error::Result};

const MAX_BUFFER_SIZE: usize = 1 << 16;

pub struct ByteStream {
    reader: opendal::Reader,

    buffer: BytesMut,
}

impl From<opendal::Reader> for ByteStream {
    fn from(reader: opendal::Reader) -> Self { Self { reader, buffer: BytesMut::new() } }
}

impl ByteStream {
    pub async fn bytes(&mut self) -> Result<Option<&[u8]>> {
        self.prepare_buffer();

        let n = self.reader.read_buf(&mut self.buffer).await.context(error::ReadFromReaderSnafu)?;

        if n == 0 {
            Ok(None)
        } else {
            Ok(Some(self.buffer.as_ref()))
        }
    }

    #[inline]
    fn prepare_buffer(&mut self) {
        let capacity = MAX_BUFFER_SIZE;

        let current_capacity = self.buffer.capacity();
        if current_capacity < capacity {
            self.buffer.reserve(capacity - current_capacity);
        }

        self.buffer.clear();
    }
}
