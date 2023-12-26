use std::io::SeekFrom;

use bytes::BytesMut;
use snafu::ResultExt;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::{error, error::Result};

const MAX_BUFFER_SIZE: usize = 1 << 16;

pub struct ByteStream {
    reader: opendal::Reader,
    start: u64,
    end: u64,
    buffer: BytesMut,
}

impl ByteStream {
    pub fn new(reader: opendal::Reader, start: u64, end: u64) -> Self {
        Self { reader, start, end, buffer: BytesMut::new() }
    }

    pub async fn bytes(&mut self) -> Result<Option<&[u8]>> {
        if self.start > self.end {
            return Ok(None);
        }

        let _ =
            self.reader.seek(SeekFrom::Start(self.start)).await.context(error::SeekReaderSnafu)?;

        self.prepare_buffer();

        let n = self
            .reader
            .read_exact(self.buffer.as_mut())
            .await
            .context(error::ReadFromReaderSnafu)?;

        if n == 0 {
            Ok(None)
        } else {
            self.start += n as u64;
            Ok(Some(self.buffer.as_ref()))
        }
    }

    #[allow(unsafe_code)]
    #[inline]
    fn prepare_buffer(&mut self) {
        let capacity = MAX_BUFFER_SIZE
            .min(usize::try_from(self.end - self.start + 1).unwrap_or(MAX_BUFFER_SIZE));

        let current_capacity = self.buffer.capacity();
        if current_capacity < capacity {
            self.buffer.reserve(capacity - current_capacity);
        }

        unsafe {
            self.buffer.set_len(capacity);
        }
    }
}
