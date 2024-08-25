mod kafka_consumer;
mod greetings;
mod observability;

use config::Config;
use dotenv::dotenv;
use opentelemetry::{global};

use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;

use serde::Deserialize;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::greetings::GreetingRepositoryImpl;
use crate::kafka_consumer::{ConsumeTopics};
use crate::observability::{init_logs, init_tracer_provider};

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


#[derive(Deserialize)]
pub(crate) struct Settings {
    pub(crate) kafka: Kafka,
    pub db: Db,
    pub otel_collector: OtelCollector
}

impl Settings {
    pub fn new() -> Self {
        dotenv().ok();

        let settings = Config::builder()
            .add_source(config::File::with_name("./res/server").required(false))
            .add_source(config::Environment::with_prefix("APP").separator("__"))
            .build()
            .unwrap();

        settings.try_deserialize().unwrap()
    }
}

#[derive(Deserialize)]
pub(crate) struct Kafka {
    pub(crate) broker: String,
    pub(crate) topic: String,
    pub(crate) consumer_group: String,
}

#[derive(Deserialize)]
pub struct Db{
    pub database_url: String
}

#[derive(Deserialize)]
pub (crate) struct OtelCollector{
    pub (crate) oltp_endpoint: String
}
