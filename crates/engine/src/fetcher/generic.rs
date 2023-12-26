use std::io::SeekFrom;

use bytes::{Bytes, BytesMut};
use snafu::ResultExt;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::{error, error::Result};

const MAX_BUFFER_SIZE: usize = 1 << 16;

pub struct ByteStream {
    reader: opendal::Reader,
    start: u64,
    end: u64,
}

impl ByteStream {
    pub const fn new(reader: opendal::Reader, start: u64, end: u64) -> Self {
        Self { reader, start, end }
    }

    pub async fn bytes(&mut self) -> Result<Option<Bytes>> {
        if self.start > self.end {
            return Ok(None);
        }

        let _ =
            self.reader.seek(SeekFrom::Start(self.start)).await.context(error::SeekReaderSnafu)?;

        let mut buf = self.new_buffer();
        let n = self.reader.read_exact(buf.as_mut()).await.context(error::ReadFromReaderSnafu)?;

        if n == 0 {
            Ok(None)
        } else {
            self.start += n as u64;
            Ok(Some(buf.freeze()))
        }
    }

    #[allow(unsafe_code)]
    #[inline]
    fn new_buffer(&self) -> BytesMut {
        let capacity = MAX_BUFFER_SIZE
            .min(usize::try_from(self.end - self.start + 1).unwrap_or(MAX_BUFFER_SIZE));
        let mut buf = BytesMut::with_capacity(capacity);
        unsafe {
            buf.set_len(capacity);
        }
        buf
    }
}
