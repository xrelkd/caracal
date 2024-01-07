pub mod config;
mod error;
mod grpc;
mod metrics;
mod web;

use std::{future::Future, net::SocketAddr, path::PathBuf, pin::Pin};

use caracal_engine::{DownloaderFactory, TaskScheduler, MINIMUM_CHUNK_SIZE};
use futures::FutureExt;
use include_dir::include_dir;
use sigfinn::{ExitStatus, LifecycleManager, Shutdown};
use snafu::ResultExt;
use tokio::{
    net::{TcpListener, UnixListener},
    task::JoinHandle,
};
use tokio_stream::wrappers::UnixListenerStream;

pub use self::{
    config::Config,
    error::{Error, Result},
};
use crate::metrics::Metrics;

pub static FRONTEND_STATIC_ASSETS_DIR: include_dir::Dir<'_> =
    include_dir!("$OUT_DIR/frontend-dist");

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
        web: web_config,
        dbus: _,
    }: Config,
) -> Result<()> {
    let lifecycle_manager = LifecycleManager::<Error>::new();

    let (task_scheduler, task_scheduler_worker) = {
        tracing::info!(
            "Setting {} as default output directory",
            task_scheduler.default_output_directory.display()
        );
        tracing::info!("Setting {} as default HTTP user-agent", task_scheduler.http.user_agent);
        tracing::info!(
            "Setting concurrent number of a task to {}",
            task_scheduler.http.concurrent_connections
        );
        let downloader_factory = DownloaderFactory::builder()
            .context(error::BuildDownloaderFactorySnafu)?
            .http_user_agent(task_scheduler.http.user_agent)
            .default_output_directory_path(task_scheduler.default_output_directory)
            .default_concurrent_number(u64::from(task_scheduler.http.concurrent_connections))
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

    let _handle = lifecycle_manager
        .spawn("Web server", create_web_server_future(web_config.listen_address, task_scheduler));

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
            tracing::info!("Listening Caracal gRPC endpoint on {}", local_socket.display());
            if let Some(local_socket_parent) = local_socket.parent() {
                if let Err(err) = tokio::fs::create_dir_all(&local_socket_parent)
                    .await
                    .context(error::CreateUnixListenerSnafu { socket_path: local_socket.clone() })
                {
                    return ExitStatus::FatalError(err);
                }
            }

            let uds_stream = match UnixListener::bind(&local_socket)
                .context(error::CreateUnixListenerSnafu { socket_path: local_socket.clone() })
            {
                Ok(uds) => UnixListenerStream::new(uds),
                Err(err) => return ExitStatus::FatalError(err),
            };

            let interceptor = grpc::Interceptor::new(grpc_access_token);
            let result = tonic::transport::Server::builder()
                .add_service(caracal_proto::SystemServer::with_interceptor(
                    grpc::SystemService::new(),
                    interceptor.clone(),
                ))
                .add_service(caracal_proto::TaskServer::with_interceptor(
                    grpc::TaskService::new(task_scheduler),
                    interceptor,
                ))
                .serve_with_incoming_shutdown(uds_stream, signal)
                .await
                .context(error::StartTonicServerSnafu);

            match result {
                Ok(()) => {
                    tracing::info!(
                        "Removing Unix domain socket `{path}`",
                        path = local_socket.display()
                    );
                    drop(tokio::fs::remove_file(local_socket).await);
                    tracing::info!("Stopped gRPC local socket server gracefully");
                    ExitStatus::Success
                }
                Err(err) => ExitStatus::Error(err),
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
            tracing::info!("Listening Caracal gRPC endpoint on {listen_address}");

            let interceptor = grpc::Interceptor::new(grpc_access_token);
            let result = tonic::transport::Server::builder()
                .add_service(caracal_proto::SystemServer::with_interceptor(
                    grpc::SystemService::new(),
                    interceptor.clone(),
                ))
                .add_service(caracal_proto::TaskServer::with_interceptor(
                    grpc::TaskService::new(task_scheduler),
                    interceptor,
                ))
                .serve_with_shutdown(listen_address, signal)
                .await
                .context(error::StartTonicServerSnafu);

            match result {
                Ok(()) => {
                    tracing::info!("Stopped gRPC HTTP server gracefully");
                    ExitStatus::Success
                }
                Err(err) => ExitStatus::Error(err),
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
            tracing::info!("Stopping Task scheduler");
            match task_scheduler.shutdown() {
                Ok(()) => tracing::info!("Stopped Task scheduler gracefully"),
                Err(err) => tracing::warn!(
                    "Error occurred while shutting down Task scheduler, error: {err}"
                ),
            }
            drop(task_scheduler_worker.await);

            ExitStatus::Success
        }
        .boxed()
    }
}

fn create_web_server_future(
    listen_address: SocketAddr,
    task_scheduler: TaskScheduler,
) -> impl FnOnce(Shutdown) -> Pin<Box<dyn Future<Output = ExitStatus<Error>> + Send>> {
    move |shutdown_signal| {
        async move {
            tracing::info!("Listening Web server on {listen_address}");

            let middleware_stack = tower::ServiceBuilder::new();

            let router = axum::Router::new()
                .merge(web::controller::api_v1_router())
                .merge(web::controller::static_assets_router())
                .layer(axum::Extension(task_scheduler))
                .layer(middleware_stack)
                .into_make_service_with_connect_info::<SocketAddr>();

            let maybe_listener =
                TcpListener::bind(&listen_address).await.context(error::BindWebServerSnafu);
            let listener = match maybe_listener {
                Ok(listener) => listener,
                Err(err) => {
                    return ExitStatus::FatalError(err);
                }
            };

            let result = axum::serve(listener, router)
                .with_graceful_shutdown(shutdown_signal)
                .await
                .context(error::ServeBindWebServerSnafu);

            match result {
                Ok(()) => {
                    tracing::info!("Stopped Web server gracefully");
                    ExitStatus::Success
                }
                Err(err) => ExitStatus::Error(err),
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
            tracing::info!("Listening metrics endpoint on {listen_address}");
            let result =
                caracal_metrics::start_metrics_server(listen_address, metrics, signal).await;
            match result {
                Ok(()) => {
                    tracing::info!("Stopped Metrics server gracefully");
                    ExitStatus::Success
                }
                Err(err) => ExitStatus::Error(Error::from(err)),
            }
        }
        .boxed()
    }
}
