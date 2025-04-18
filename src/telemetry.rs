use tokio::task::JoinHandle;
use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Registry};

/// # `get_subscriber`
///
/// Compose multiple layesr into a `tracing`'s subscriber
///
/// NOTE: We are using `impl Subscriber` as return type to avoid
/// having to spell out the actual type of the returned subscriber,
/// which is indeed quite complex.
/// We need to explicitly call out that the returned subscriber is
/// `Send` and `Sync` to make it possible to pass it to `init_subscriber`
/// later on.
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // Print all spans at info-level or above if RUST_LOG hasn't been set.
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name, sink);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// # `init_subscriber`
///
/// Registers a subscriber as global default to process span data.
///
/// NOTE: It should only be called once.
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber.");
}

/// # spawn_blocking_with_tracing
///
/// Spawn a blocking thread, necessary when dealing with a CPU intensive task.
/// NOTE: Spawning a non blocking thread in a span doesn't allow me to track informations involving
/// this span across threads, this means I will need to attach the span of the task that
/// is calling `spawn_blocking` to the new spawned task.
pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    // we pass the ownership of `current_span` to the closure
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}
