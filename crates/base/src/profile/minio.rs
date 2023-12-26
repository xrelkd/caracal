#[derive(Clone, Debug)]
pub struct MinioAlias {
    pub endpoint_url: http::Uri,

    pub access_key: String,

    pub secret_key: String,
}

#[derive(Clone, Debug, Default)]
pub struct MinioPath {
    pub alias: String,

    pub bucket: String,

    pub object: String,
}
