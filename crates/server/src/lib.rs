pub mod config;
mod error;
mod grpc;
mod metrics;

use std::{future::Future, net::SocketAddr, path::PathBuf, pin::Pin};

use caracal_engine::{DownloaderFactory, TaskScheduler};
use futures::FutureExt;
use sigfinn::{ExitStatus, LifecycleManager, Shutdown};
use snafu::ResultExt;
use tokio::{net::UnixListener, task::JoinHandle};
use tokio_stream::wrappers::UnixListenerStream;

pub use self::{
    config::Config,
    error::{Error, Result},
};
use crate::metrics::Metrics;

const MINIMUM_CHUNK_SIZE: u64 = 100 * 1024;

/// # Errors
///
/// This function will return an error if the server fails to start.
pub async fn serve_with_shutdown(
    Config {
        task_scheduler,
        ssh_servers,
        minio_aliases,
        grpc_listen_address,
        grpc_local_socket,
        grpc_access_token,
        metrics: metrics_config,
        dbus: _,
    }: Config,
) -> Result<()> {
    let lifecycle_manager = LifecycleManager::<Error>::new();

    let (task_scheduler, task_scheduler_worker) = {
        let downloader_factory = DownloaderFactory::builder()
            .http_user_agent(task_scheduler.http.user_agent)
            .default_worker_number(u64::from(task_scheduler.http.concurrent_connections))
            .minimum_chunk_size(MINIMUM_CHUNK_SIZE)
            .ssh_servers(ssh_servers)
            .minio_aliases(minio_aliases)
            .build()
            .context(error::InitializeDownloaderSnafu)?;

        TaskScheduler::new(downloader_factory, task_scheduler.concurrent_number)
    };
    let _handle = lifecycle_manager.spawn(
        "Task Scheduler",
        create_task_scheduler_future(task_scheduler.clone(), task_scheduler_worker),
    );

    if let Some(grpc_listen_address) = grpc_listen_address {
        let _handle = lifecycle_manager.spawn(
            "gRPC HTTP server",
            create_grpc_http_server_future(
                grpc_listen_address,
                grpc_access_token.clone(),
                task_scheduler.clone(),
            ),
        );
    }

    if let Some(grpc_local_socket) = grpc_local_socket {
        let _handle = lifecycle_manager.spawn(
            "gRPC local socket server",
            create_grpc_local_socket_server_future(
                grpc_local_socket,
                grpc_access_token,
                task_scheduler.clone(),
            ),
        );
    }

    if metrics_config.enable {
        let metrics = Metrics::new()?;

        let _handle = lifecycle_manager.spawn(
            "Metrics server",
            create_metrics_server_future(metrics_config.listen_address, metrics),
        );
    }

    if let Ok(Err(err)) = lifecycle_manager.serve().await {
        tracing::error!("{err}");
        Err(err)
    } else {
        Ok(())
    }
}

fn create_grpc_local_socket_server_future(
    local_socket: PathBuf,
    grpc_access_token: Option<String>,
    task_scheduler: TaskScheduler,
) -> impl FnOnce(Shutdown) -> Pin<Box<dyn Future<Output = ExitStatus<Error>> + Send>> {
    move |signal| {
        async move {
            tracing::info!("Listen Caracal gRPC endpoint on {}", local_socket.display());
            if let Some(local_socket_parent) = local_socket.parent() {
                if let Err(err) = tokio::fs::create_dir_all(&local_socket_parent)
                    .await
                    .context(error::CreateUnixListenerSnafu { socket_path: local_socket.clone() })
                {
                    return ExitStatus::Failure(err);
                }
            }

            let uds_stream = match UnixListener::bind(&local_socket)
                .context(error::CreateUnixListenerSnafu { socket_path: local_socket.clone() })
            {
                Ok(uds) => UnixListenerStream::new(uds),
                Err(err) => return ExitStatus::Failure(err),
            };

            // TODO: put task_scheduler into grpc service
            drop(task_scheduler);

            let interceptor = grpc::Interceptor::new(grpc_access_token);
            let result = tonic::transport::Server::builder()
                .add_service(caracal_proto::SystemServer::with_interceptor(
                    grpc::SystemService::new(),
                    interceptor.clone(),
                ))
                .serve_with_incoming_shutdown(uds_stream, signal)
                .await
                .context(error::StartTonicServerSnafu);

            match result {
                Ok(()) => {
                    tracing::info!(
                        "Remove Unix domain socket `{path}`",
                        path = local_socket.display()
                    );
                    drop(tokio::fs::remove_file(local_socket).await);
                    tracing::info!("gRPC local socket server is shut down gracefully");
                    ExitStatus::Success
                }
                Err(err) => ExitStatus::Failure(err),
            }
        }
        .boxed()
    }
}

fn create_grpc_http_server_future(
    listen_address: SocketAddr,
    grpc_access_token: Option<String>,
    task_scheduler: TaskScheduler,
) -> impl FnOnce(Shutdown) -> Pin<Box<dyn Future<Output = ExitStatus<Error>> + Send>> {
    move |signal| {
        async move {
            tracing::info!("Listen Caracal gRPC endpoint on {listen_address}");

            // TODO: put task_scheduler into grpc service
            drop(task_scheduler);

            let interceptor = grpc::Interceptor::new(grpc_access_token);
            let result = tonic::transport::Server::builder()
                .add_service(caracal_proto::SystemServer::with_interceptor(
                    grpc::SystemService::new(),
                    interceptor.clone(),
                ))
                .serve_with_shutdown(listen_address, signal)
                .await
                .context(error::StartTonicServerSnafu);

            match result {
                Ok(()) => {
                    tracing::info!("gRPC HTTP server is shut down gracefully");
                    ExitStatus::Success
                }
                Err(err) => ExitStatus::Failure(err),
            }
        }
        .boxed()
    }
}

fn create_task_scheduler_future(
    task_scheduler: TaskScheduler,
    task_scheduler_worker: JoinHandle<()>,
) -> impl FnOnce(Shutdown) -> Pin<Box<dyn Future<Output = ExitStatus<Error>> + Send>> {
    move |shutdown_signal| {
        async move {
            shutdown_signal.await;
            let result = task_scheduler.shutdown();
            drop(task_scheduler_worker.await);
            match result {
                Ok(()) => {
                    tracing::info!("Task scheduler is shut down gracefully");
                    ExitStatus::Success
                }
                Err(err) => {
                    tracing::warn!(
                        "Error occurred while shutting down Task scheduler, error: {err}"
                    );
                    ExitStatus::Success
                }
            }
        }
        .boxed()
    }
}

fn create_metrics_server_future<Metrics>(
    listen_address: SocketAddr,
    metrics: Metrics,
) -> impl FnOnce(Shutdown) -> Pin<Box<dyn Future<Output = ExitStatus<Error>> + Send>>
where
    Metrics: caracal_metrics::Metrics + 'static,
{
    move |signal| {
        async move {
            tracing::info!("Listen metrics endpoint on {listen_address}");
            let result =
                caracal_metrics::start_metrics_server(listen_address, metrics, signal).await;
            match result {
                Ok(()) => {
                    tracing::info!("Metrics server is shut down gracefully");
                    ExitStatus::Success
                }
                Err(err) => ExitStatus::Failure(Error::from(err)),
            }
        }
        .boxed()
    }
}
