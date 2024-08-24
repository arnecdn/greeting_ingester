mod kafka_consumer;
mod greetings;

use config::Config;
use dotenv::dotenv;
use once_cell::sync::Lazy;
use opentelemetry::{global, KeyValue};
use opentelemetry::logs::LogError;
use opentelemetry::propagation::Extractor;
use opentelemetry::trace::TraceError;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::{Resource, runtime};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::TracerProvider;
use rdkafka::message::{BorrowedHeaders, Headers};
use serde::Deserialize;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::greetings::GreetingRepositoryImpl;
use crate::kafka_consumer::{ConsumeTopics};


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
    consumer.consume_and_store(repo).await.expect("Failed starting subscription...")
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

static RESOURCE: Lazy<Resource> = Lazy::new(|| {
    Resource::new(vec![KeyValue::new(
        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
        "greeting_processor_rust",
    )])
});

fn init_logs(otlp_endpoint: &str) -> Result<LoggerProvider, LogError> {
    opentelemetry_otlp::new_pipeline()
        .logging()
        .with_resource(RESOURCE.clone())
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(otlp_endpoint),
        )
        .install_batch(runtime::Tokio)
}

fn init_tracer_provider(otlp_endpoint: &str) -> Result<TracerProvider, TraceError> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(otlp_endpoint),
        )
        .with_trace_config(opentelemetry_sdk::trace::Config::default().with_resource(RESOURCE.clone()))
        .install_batch(runtime::Tokio)
}

pub struct HeaderExtractor<'a>(pub &'a BorrowedHeaders);

impl<'a> Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        for i in 0..self.0.count() {
            if let Ok(val) = self.0.get_as::<str>(i) {
                if val.key == key {
                    return val.value
                }
            }
        }
        None
    }

    fn keys(&self) -> Vec<&str> {
        self.0.iter().map(|kv| kv.key).collect::<Vec<_>>()
    }
}