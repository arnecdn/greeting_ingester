
use async_trait::async_trait;
use log::{info, warn};
use opentelemetry::{global, Key, KeyValue, StringValue};
use opentelemetry::trace::{Span, SpanKind, Status, Tracer};
use rdkafka::{ClientConfig, ClientContext, Message, TopicPartitionList};
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance, StreamConsumer};
use rdkafka::error::{KafkaError, KafkaResult};
use rdkafka::message::Headers;

use crate::{HeaderExtractor, Settings};
use crate::greetings::{GreetingRepository, GreetingRepositoryImpl, RepoError};

struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        info!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        info!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        info!("Committing offsets: {:?}", result);
    }
}

// A type alias with your custom consumer can be created for convenience.
type LoggingConsumer = StreamConsumer<CustomContext>;

#[derive(Debug)]
pub struct ConsumerError {
    err_msg: String,
}

impl From<RepoError> for ConsumerError {
    fn from(value: RepoError) -> Self {
        ConsumerError { err_msg: value.error_message }
    }
}

impl From<KafkaError> for ConsumerError {
    fn from(value: KafkaError) -> Self {
        ConsumerError { err_msg: value.to_string() }
    }
}

pub struct KafkaConsumer {
    // config: Settings,
    topic: String,
    consumer: LoggingConsumer
}

impl KafkaConsumer {
    pub async fn new(settings: Settings) -> Result<Self, ConsumerError> {
        return Ok(Self {
            topic: settings.kafka.topic,
            consumer:ClientConfig::new()
                .set("group.id", settings.kafka.consumer_group)
                .set("bootstrap.servers", settings.kafka.broker)
                .set("enable.partition.eof", "false")
                .set("session.timeout.ms", "6000")
                .set("enable.auto.commit", "false")
                .set_log_level(RDKafkaLogLevel::Debug)
                .create_with_context(CustomContext).expect("Failed creating consumer")

        })
    }
}

#[async_trait]
pub trait ConsumeTopics {
    async fn consume_and_store(&self, repo: Box<GreetingRepositoryImpl>) -> Result<(), ConsumerError>;
}


#[async_trait]
impl ConsumeTopics for KafkaConsumer {
    async fn consume_and_store(&self, mut repo: Box<GreetingRepositoryImpl>) -> Result<(), ConsumerError> {

        self.consumer
            .subscribe(&[&self.topic])?;

        info!("Starting to subscriobe on topic: {}", &self.topic);
        // let mut repo = GreetingRepositoryImpl::new(self.config.db.database_url.clone()).await?;

        loop {
            match &self.consumer.recv().await {
                Err(e) => warn!("Kafka error: {}", e),
                Ok(m) => {
                    let payload = match m.payload_view::<str>() {
                        None => "",
                        Some(Ok(s)) => s,
                        Some(Err(e)) => {
                            warn!("Error while deserializing message payload: {:?}", e);
                            ""
                        }
                    };
                    info!("topic: {}, partition: {}, offset: {}, timestamp: {:?}, payload: '{}'",
                    m.topic(), m.partition(), m.offset(), m.timestamp(), payload,);
                    let msg = serde_json::from_str(&payload).unwrap();
                    repo.store(msg).await?;
                    if let Some(headers) = m.headers() {
                        for header in headers.iter() {
                            info!("  Header {:#?}: {:?}", header.key, header.value);
                        }

                        let context = global::get_text_map_propagator(|propagator| {
                            propagator.extract(&HeaderExtractor(headers))
                        });

                        let mut span =
                            global::tracer("consumer").start_with_context("consume_payload", &context);

                        span.set_status(Status::Ok);
                        span.end();
                    }
                    self.consumer.commit_message(&m, CommitMode::Async).unwrap();

                }
            };
        }
    }
}