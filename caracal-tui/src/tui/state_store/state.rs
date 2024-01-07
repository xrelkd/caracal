use caracal_base::model;

/// State holds the state of the application
#[derive(Debug, Clone, Default)]
pub struct State {
    task_statuses: Vec<model::TaskStatus>,

    server_endpoint: http::Uri,

    access_token: Option<String>,

    daemon_version: Option<semver::Version>,

    timer: usize,
}

impl State {
    pub const fn server_endpoint(&self) -> &http::Uri { &self.server_endpoint }

    pub fn set_server_endpoint(&mut self, server_endpoint: http::Uri) {
        self.server_endpoint = server_endpoint;
    }

    pub fn access_token(&self) -> Option<&str> { self.access_token.as_deref() }

    pub fn set_access_token(&mut self, access_token: Option<String>) {
        self.access_token = access_token;
    }

    pub const fn task_statuses(&self) -> &Vec<model::TaskStatus> { &self.task_statuses }

    pub fn set_task_statuses(&mut self, task_statuses: Vec<model::TaskStatus>) {
        self.task_statuses = task_statuses;
    }

    pub const fn daemon_version(&self) -> Option<&semver::Version> { self.daemon_version.as_ref() }

    pub fn set_daemon_version(&mut self, daemon_version: semver::Version) {
        self.daemon_version = Some(daemon_version);
    }

    pub fn mark_disconnected(&mut self) { self.daemon_version = None; }

    pub fn tick_timer(&mut self) { self.timer += 1; }
}
