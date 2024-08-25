mod kafka_consumer;
mod greetings;
mod observability;
mod settings;

use opentelemetry::{global};

use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::greetings::GreetingRepositoryImpl;
use crate::kafka_consumer::{ConsumeTopics};
use crate::observability::{init_logs, init_tracer_provider};
use crate::settings::Settings;

#[tokio::main]
async fn main() {

    let app_config = Settings::new();
    let result = init_tracer_provider(&app_config.otel_collector.oltp_endpoint);
    let tracer_provider = result.unwrap();
    global::set_tracer_provider(tracer_provider.clone());

    // Initialize logs and save the logger_provider.
    let logger_provider = init_logs(&app_config.otel_collector.oltp_endpoint).unwrap();

    // Create a new OpenTelemetryTracingBridge using the above LoggerProvider.
    let layer = OpenTelemetryTracingBridge::new(&logger_provider);
    tracing_subscriber::registry()
        .with(layer)
        .init();

    let repo = Box::new(GreetingRepositoryImpl::new(app_config.db.database_url.clone()).await.expect("failed"));
    let consumer = kafka_consumer::KafkaConsumer::new(app_config).await.expect("Failed to create kafka consumer");
    consumer.consume_and_store(repo).await.expect("Failed starting subscription...");
    global::shutdown_tracer_provider();
    logger_provider.shutdown().expect("Failed shutting down loggprovider");
}

