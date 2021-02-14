use mime::Mime;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MimeData {
    mime: Mime,
    data: Vec<u8>,
}

impl Default for MimeData {
    fn default() -> MimeData { MimeData { mime: mime::TEXT_PLAIN_UTF_8, data: Default::default() } }
}

impl MimeData {
    #[inline]
    pub fn new(mime: Mime, data: Vec<u8>) -> MimeData { MimeData { mime, data } }

    #[inline]
    pub fn bytes(&self) -> &[u8] { self.data.as_ref() }

    #[inline]
    pub fn clone_bytes(&self) -> Vec<u8> { self.data.clone() }

    #[inline]
    pub fn data_mut(&mut self) -> &[u8] { self.data.as_ref() }

    #[inline]
    pub fn mime(&self) -> &Mime { &self.mime }

    #[inline]
    pub fn len(&self) -> usize { self.data.len() }

    #[inline]
    pub fn is_empty(&self) -> bool { self.data.is_empty() }

    #[inline]
    pub fn destruct(self) -> (Mime, Vec<u8>) { (self.mime, self.data) }
}
