mod kafka_consumer;
mod greetings;
mod settings;
mod db;

use futures_util::join;
use opentelemetry::{global};
use opentelemetry::trace::TracerProvider;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use sqlx::Pool;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::greetings::{GreetingRepositoryImpl};
use crate::kafka_consumer::{ConsumeTopics};
use crate::settings::Settings;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_config = Settings::new();

    greeting_otel::init_otel(&app_config.otel_collector.oltp_endpoint, "greeting_processor",&app_config.kube.my_pod_name).await;

    let pool = Box::new(db::init_db(app_config.db.database_url.clone()).await.expect("Expected db pool"));
    let repo = Box::new(GreetingRepositoryImpl::new(pool.clone()).await.expect("failed"));
    let mut consumer = kafka_consumer::KafkaConsumer::new(app_config, repo).await.expect("Failed to create kafka consumer");

    let consumer_handle = consumer.consume_and_store();

    join!(consumer_handle);

    global::shutdown_tracer_provider();
    Ok(())
}


