use chrono::Local;
use derive_more::Display;
use futures_util::StreamExt;
use log::info;
use rdkafka::{ClientConfig, Message};
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::error::KafkaError;
use crate::{ Settings};

async fn consume_single()->Result<(), GreetingProcessorError> {
    let app_config = Settings::new();
    let mut kafka_config: ClientConfig = ClientConfig::new();
    kafka_config.set("bootstrap.servers", app_config.kafka.broker);
    kafka_config.set("group.id", app_config.kafka.consumer_group);
    ;
    // put here to show that the microservice has started
    // println!("Datetime:{},Started...", Local::now());

    let consumer: StreamConsumer = kafka_config.create()?;

    consumer.subscribe(&[&*app_config.kafka.topic])?;

    let borrowed_msg = consumer.stream().next().await.unwrap()?;

    let payload = match borrowed_msg.payload_view::<str>() {
        None => "Payload ",
        Some(Ok(s)) => s,
        Some(Err(e)) => return Err(GreetingProcessorError{msg:e.to_string()})
    };

    info!("partition: {}, offset: {}, payload: '{}'",
              borrowed_msg.partition(), borrowed_msg.offset(), payload);

    consumer.commit_message(&borrowed_msg, CommitMode::Sync)?;//.expect(&format!("Panic: not able to commit offset:{}", borrowed_msg.offset()));

    info!("committed  offset: {} on partition: {}'",borrowed_msg.offset(), borrowed_msg.partition(), );
    Ok(())
}
#[derive(Debug, Display)]
pub struct GreetingProcessorError{
    msg: String
}

impl From<KafkaError> for  GreetingProcessorError {
    fn from(value: KafkaError) -> Self {
        Self{msg:value.to_string()}
    }
}
