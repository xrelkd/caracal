pub use crate::error::Error;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Chunk {
    pub start: u64,

    pub end: u64,

    pub received: u64,
}

impl Chunk {
    pub const fn len(&self) -> u64 { self.end - self.start + 1 }

    pub const fn remaining(&self) -> u64 {
        let len = self.len();
        debug_assert!(len >= self.received);
        len - self.received
    }

    pub fn split(&mut self) -> Option<Self> {
        if self.received == self.len() {
            None
        } else {
            let len = self.remaining() / 2;
            let new_chunk = Self { start: self.start + len, end: self.end, received: 0 };
            self.end = self.start + len - 1;
            Some(new_chunk)
        }
    }

    pub fn freeze(&mut self) -> Option<Self> {
        if self.received == 0 {
            None
        } else {
            let new_chunk = Self { start: self.start + self.received, end: self.end, received: 0 };
            self.end = self.start + self.received - 1;
            Some(new_chunk)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Chunk;

    #[test]
    fn test_split() {
        let len = 2048;
        let mut origin_chunk = Chunk { start: 0, end: len - 1, received: 0 };
        assert_eq!(len, origin_chunk.len());
        let new_chunk = origin_chunk.split().unwrap();
        assert_eq!(new_chunk, Chunk { start: 1024, end: len - 1, received: 0 });
        assert_eq!(len, origin_chunk.len() + new_chunk.len());
        assert_eq!(1024, origin_chunk.len());
        assert_eq!(1024, new_chunk.len());

        let len = 1001;
        let mut origin_chunk = Chunk { start: 0, end: len - 1, received: 0 };
        assert_eq!(len, origin_chunk.len());
        let new_chunk = origin_chunk.split().unwrap();
        assert_eq!(new_chunk, Chunk { start: 500, end: 1000, received: 0 });
        assert_eq!(len, origin_chunk.len() + new_chunk.len());
        assert_eq!(500, origin_chunk.len());
        assert_eq!(501, new_chunk.len());
    }

    #[test]
    fn test_freeze() {
        let len = 2048;
        let mut origin_chunk = Chunk { start: 0, end: len - 1, received: 0 };
        assert_eq!(len, origin_chunk.len());
        let new_chunk = origin_chunk.freeze();
        assert!(new_chunk.is_none());

        let len = 2048;
        let received = 20;
        let mut origin_chunk = Chunk { start: 0, end: len - 1, received };
        let new_chunk = origin_chunk.freeze().unwrap();
        assert_eq!(len, origin_chunk.len() + new_chunk.len());
        assert_eq!(received, origin_chunk.len());
        assert_eq!(len - received, new_chunk.len());

        let len = 2047;
        let received = 20;
        let mut origin_chunk = Chunk { start: 0, end: len - 1, received };
        let new_chunk = origin_chunk.freeze().unwrap();
        assert_eq!(len, origin_chunk.len() + new_chunk.len());
        assert_eq!(received, origin_chunk.len());
        assert_eq!(len - received, new_chunk.len());
    }
}
