mod kafka_consumer;
mod greetings;
mod open_telemetry;
mod settings;
mod greeting_log;
mod db;

use std::thread;
use std::time::Duration;
use actix_web::{web, App, HttpServer};
use futures_util::join;
use log::{info};
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

    let (logger_provider,_) = init_otel(&app_config).await;
    // let meter_provider = init_metrics(&app_config.otel_collector.oltp_endpoint).expect("Failed initializing metrics");
    // global::set_meter_provider(meter_provider);

    let pool = Box::new(db::init_db(app_config.db.database_url.clone()).await.expect("Expected db pool"));
    let repo = Box::new(GreetingRepositoryImpl::new(pool.clone()).await.expect("failed"));
    let mut consumer = kafka_consumer::KafkaConsumer::new(app_config, repo).await.expect("Failed to create kafka consumer");

    let log_generator_handle = greeting_log::generate_logg(pool.clone());
    let consumer_handle = consumer.consume_and_store();
    let server_handle = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(greeting_log::list_log_entries)

    })
        .bind(("127.0.0.1", 8080))?
        .run();

    join!(consumer_handle, log_generator_handle, server_handle);

    global::shutdown_tracer_provider();
    logger_provider.shutdown();
    Ok(())
}

async fn init_otel(app_config: &Settings) -> (LoggerProvider, opentelemetry_sdk::trace::TracerProvider) {
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

    (logger_provider, tracer_provider)
}


