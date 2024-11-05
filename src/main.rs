mod kafka_consumer;
mod greetings;
mod open_telemetry;
mod settings;

use std::thread;
use std::time::Duration;
use futures_util::task::SpawnExt;
use opentelemetry::{global};
use opentelemetry::trace::TracerProvider;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::greetings::{GreetingRepository, GreetingRepositoryImpl};
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
    let pool = Box::new(greetings::create_pool(app_config.db.database_url.clone()).await.expect("Failed establishing db pool"));
    let logg_generator_handle = tokio::task::spawn(greetings::generate_logg(pool.clone()));
    let repo = Box::new(GreetingRepositoryImpl::new(pool.clone()).await.expect("failed"));

    let mut consumer = kafka_consumer::KafkaConsumer::new(app_config, repo).await.expect("Failed to create kafka consumer");
    let kafka_consumer_handle = consumer.consume_and_store(); //.await.expect("Failed starting subscription...");
    tokio::join!(logg_generator_handle, kafka_consumer_handle);
    global::shutdown_tracer_provider();
    logger_provider.shutdown().expect("Failed shutting down loggprovider");
}



