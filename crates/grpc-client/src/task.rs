use async_trait::async_trait;

// use crate::error::AddUriError;
// use caracal_proto as proto;
// use tonic::Request;
use crate::Client;

#[async_trait]
pub trait Task {}

#[async_trait]
impl Task for Client {}
