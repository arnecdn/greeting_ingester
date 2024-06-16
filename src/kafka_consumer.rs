use rdkafka::{ClientConfig, ClientContext, Message, TopicPartitionList};
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance, StreamConsumer};
use rdkafka::error::KafkaResult;
use log::{info, warn};
use rdkafka::message::Headers;
use crate::greetings::{GreetingRepository, GreetingRepositoryImpl, RepoError};
use crate::{greetings, Settings};

struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self,  rebalance: &Rebalance) {
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
pub struct ConsumerError{
    err_msg: String
}

impl From<RepoError> for ConsumerError{
    fn from(value: RepoError) -> Self {
        ConsumerError{ err_msg: value.error_message}
    }
}
pub async fn consume_and_print() -> Result<(),ConsumerError>{

    let context = CustomContext;
    let app_config = Settings::new();
    let consumer: LoggingConsumer = ClientConfig::new()
        .set("group.id", app_config.kafka.consumer_group)
        .set("bootstrap.servers", app_config.kafka.broker)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        //.set("statistics.interval.ms", "30000")
        //.set("auto.offset.reset", "smallest")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create_with_context(context)
        .expect("Consumer creation failed");

    let mut repo = GreetingRepositoryImpl::new(app_config.db.database_url).await?;
    consumer
        .subscribe(&[&app_config.kafka.topic])
        .expect("Can't subscribe to specified topics");

    info!("Starting to subscriobe on topic: {}", &app_config.kafka.topic);

    loop {
        match consumer.recv().await {
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
                }
                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        };
    }
}