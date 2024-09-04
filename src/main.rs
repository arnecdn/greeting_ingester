mod kafka_consumer;
mod greetings;
mod open_telemetry;
mod settings;

use opentelemetry::{global};
use opentelemetry::trace::TracerProvider;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::greetings::GreetingRepositoryImpl;
use crate::kafka_consumer::{ConsumeTopics};
// use crate::open_telemetry::{init_logs, init_metrics, init_tracer_provider};
use crate::settings::Settings;

#[tokio::main]
async fn main() {

    let app_config = Settings::new();
    let result = open_telemetry::init_tracer_provider(&app_config.otel_collector.oltp_endpoint);
    let tracer_provider = result.unwrap();
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Create a tracing layer with the configured tracer
    let tracer_layer = tracing_opentelemetry::layer().
        with_tracer(tracer_provider.tracer("greeting_rust"));

    // Initialize logs and save the logger_provider.
    let logger_provider = open_telemetry::init_logs(&app_config.otel_collector.oltp_endpoint).unwrap();
    // Create a new OpenTelemetryTracingBridge using the above LoggerProvider.
    let logger_layer = OpenTelemetryTracingBridge::new(&logger_provider);

    let filter = EnvFilter::new("info")
        .add_directive("hyper=info".parse().unwrap())
        .add_directive("h2=info".parse().unwrap())
        .add_directive("tonic=info".parse().unwrap())
        .add_directive("reqwest=info".parse().unwrap());

    tracing_subscriber::registry()
        .with(logger_layer)
        .with(filter)
        .with(tracer_layer)
        .init();

    // let meter_provider = init_metrics(&app_config.otel_collector.oltp_endpoint).expect("Failed initializing metrics");
    // global::set_meter_provider(meter_provider);


    let repo = Box::new(GreetingRepositoryImpl::new(app_config.db.database_url.clone()).await.expect("failed"));
    let mut consumer = kafka_consumer::KafkaConsumer::new(app_config, repo).await.expect("Failed to create kafka consumer");
    consumer.consume_and_store().await.expect("Failed starting subscription...");
    global::shutdown_tracer_provider();
    logger_provider.shutdown().expect("Failed shutting down loggprovider");
}

