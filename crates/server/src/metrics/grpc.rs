use std::sync::LazyLock;

use prometheus::IntCounter;

pub static REQUESTS_TOTAL: LazyLock<IntCounter> = LazyLock::new(|| {
    IntCounter::new("grpc_requests_total", "Total number of request from gRPC")
        .expect("setup metrics")
});
