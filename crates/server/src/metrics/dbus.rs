use std::sync::LazyLock;

use prometheus::{Histogram, HistogramOpts, IntCounter};

pub static REQUESTS_TOTAL: LazyLock<IntCounter> = LazyLock::new(|| {
    IntCounter::new("dbus_requests_total", "Total number of request from D-Bus")
        .expect("setup metrics")
});

pub static REQUEST_DURATION_SECONDS: LazyLock<Histogram> = LazyLock::new(|| {
    Histogram::with_opts(HistogramOpts::new(
        "dbus_request_duration_seconds",
        "Latencies of handling request with D-Bus in seconds",
    ))
    .expect("setup metrics")
});
