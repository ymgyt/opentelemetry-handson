use opentelemetry::sdk::metrics::controllers::BasicController;
use opentelemetry_otlp::WithExportConfig;
use tracing::{error, info, info_span};
use tracing_futures::Instrument;

// https://github.com/open-telemetry/opentelemetry-rust/blob/d4b9befea04bcc7fc19319a6ebf5b5070131c486/examples/basic-otlp/src/main.rs#L35-L52
fn build_metrics_controller() -> BasicController {
    opentelemetry_otlp::new_pipeline()
        .metrics(
            opentelemetry::sdk::metrics::selectors::simple::histogram(Vec::new()),
            opentelemetry::sdk::export::metrics::aggregation::cumulative_temporality_selector(),
            opentelemetry::runtime::Tokio,
        )
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .build()
        .expect("Failed to build metrics controller")
}

fn init_tracing() {
    // Configure otel exporter.
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(
            opentelemetry::sdk::trace::config()
                .with_sampler(opentelemetry::sdk::trace::Sampler::AlwaysOn)
                .with_id_generator(opentelemetry::sdk::trace::RandomIdGenerator::default())
                .with_resource(opentelemetry::sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", "sample-app"),
                ])),
        )
        .install_batch(opentelemetry::runtime::Tokio)
        // .install_simple()
        .expect("Not running in tokio runtime");

    // Compatible layer with tracing.
    let otel_trace_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let otel_metrics_layer = tracing_opentelemetry::MetricsLayer::new(build_metrics_controller());

    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    tracing_subscriber::Registry::default()
        .with(tracing_subscriber::fmt::Layer::new().with_ansi(true))
        .with(otel_trace_layer)
        .with(otel_metrics_layer)
        .with(tracing_subscriber::filter::LevelFilter::INFO)
        .init();
}

async fn start() {
    let req_id = "123";
    let user = "ymgyt";

    operation(req_id, user).await;
}

#[tracing::instrument(
    fields(?request_id)
)]
async fn operation(request_id: &str, user: &str) {
    // trace
    // https://docs.rs/tracing-opentelemetry/latest/tracing_opentelemetry/struct.MetricsLayer.html#usage
    info!(ops = "xxx", "message");

    let res = reqwest::get("http://localhost:8000/foo").await.unwrap();
    info!("{res:#?}");
}

#[tokio::main]
async fn main() {
    opentelemetry::global::set_text_map_propagator(
        opentelemetry::sdk::propagation::TraceContextPropagator::new(),
    );

    init_tracing();

    let version = env!("CARGO_PKG_VERSION");

    start().instrument(info_span!("request", %version)).await;

    opentelemetry::global::shutdown_tracer_provider();

    // tokio::time::sleep(std::time::Duration::from_secs(60)).await;
}
