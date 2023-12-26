#[derive(Clone, Debug)]
pub struct SshConfig {
    pub endpoint: String,

    pub user: String,

    pub identity_file: String,
}
