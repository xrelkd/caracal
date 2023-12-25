use std::path::PathBuf;

use reqwest::StatusCode;
use snafu::Snafu;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Connection timed out"))]
    ConnectionTimedOut,

    #[snafu(display("Destination file `{}` already exists", file_path.display()))]
    DestinationFileExists { file_path: PathBuf },

    #[snafu(display("Fetching directory is not supported"))]
    FetchingDirectory,

    #[snafu(display("The chunk size is invalid, value: {value}"))]
    BadChunkSize { value: u64 },

    #[snafu(display("URI {uri} is not a valid MinIO URL"))]
    InvalidMinioUrl { uri: http::Uri },

    #[snafu(display("Hostname is not a provided"))]
    HostnameNotProvided,

    #[snafu(display("Resource not found, URI: {uri}"))]
    NotFound { uri: http::Uri },

    #[snafu(display("The scheme `{scheme}` is not supported"))]
    UnsupportedScheme { scheme: String },

    #[snafu(display("Unknown HTTP error, status code: {status_code}"))]
    UnknownHttpError { status_code: StatusCode },

    #[snafu(display("Could not parse length from HTTP header, value: {value}, error: {source}"))]
    ParseLengthFromHttpHeader { value: String, source: std::num::ParseIntError },

    #[snafu(display("Error occurs while building OpenDAL operator, error: {source}"))]
    BuildOpenDALOperator { source: opendal::Error },

    #[snafu(display("Error occurs while opening file `{}`, error: {source}", file_path.display()))]
    OpenFile { file_path: PathBuf, source: std::io::Error },

    #[snafu(display("Error occurs while creating file `{}`, error: {source}", file_path.display()))]
    CreateFile { file_path: PathBuf, source: std::io::Error },

    #[snafu(display("Error occurs while creating control file `{}`, error: {source}", file_path.display()))]
    CreateControlFile { file_path: PathBuf, source: std::io::Error },

    #[snafu(display("Error occurs while writing file `{}`, error: {source}", file_path.display()))]
    WriteFile { file_path: PathBuf, source: std::io::Error },

    #[snafu(display("Error occurs while flushing file `{}`, error: {source}", file_path.display()))]
    FlushFile { file_path: PathBuf, source: std::io::Error },

    #[snafu(display("Error occurs while seeking file `{}`, error: {source}", file_path.display()))]
    SeekFile { file_path: PathBuf, source: std::io::Error },

    #[snafu(display("Error occurs while resizing file `{}`, error: {source}", file_path.display()))]
    ResizeFile { file_path: PathBuf, source: std::io::Error },

    #[snafu(display("Error occurs while creating reader, error: {source}"))]
    CreateReader { source: opendal::Error },

    #[snafu(display("Error occurs while reading from reader, error: {source}"))]
    ReadFromReader { source: std::io::Error },

    #[snafu(display("Error occurs while seeking in reader, error: {source}"))]
    SeekReader { source: std::io::Error },

    #[snafu(display("Error occurs while cloning file instance `{}`, error: {source}", file_path.display()))]
    CloneFileInstance { file_path: PathBuf, source: std::io::Error },

    #[snafu(display("Error occurs while fetching range via HTTP, error: {source}"))]
    FetchRangeFromHttp { source: reqwest::Error },

    #[snafu(display("Error occurs while fetching bytes from HTTP, error: {source}"))]
    FetchBytesFromHttp { source: reqwest::Error },

    #[snafu(display("Error occurs while fetching bytes from MinIO, error: {source}"))]
    FetchBytesFromMinio { source: opendal::Error },

    #[snafu(display("Error occurs while getting metadata from SFTP server, error: {source}"))]
    GetMetadataFromSftp { source: opendal::Error },

    #[snafu(display("Error occurs while fetching HTTP header, error: {source}"))]
    FetchHttpHeader { source: reqwest::Error },

    #[snafu(display("Failed to get length of file `{}`, error: {source}", file_path.display()))]
    GetFileLength { file_path: PathBuf, source: std::io::Error },

    #[snafu(display("Error occurs while getting metadata from MinIO, error: {source}"))]
    GetMetadataFromMinio { source: opendal::Error },

    #[snafu(display("Error occurs while getting metadata from file system, error: {source}"))]
    GetMetadataFromFileSystem { source: opendal::Error },

    #[snafu(display("MinIO alias `{alias}` not found"))]
    MinioAliasNotFound { alias: String },

    #[snafu(display("SSH configuration `{endpoint}` not found"))]
    SshConfigNotFound { endpoint: String },

    #[snafu(display("Error occurs while join tokio task, error: {source}"))]
    JoinTask { source: tokio::task::JoinError },
}
