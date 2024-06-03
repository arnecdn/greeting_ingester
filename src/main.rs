use chrono::{Local, NaiveDateTime};
use config::Config;
use dotenv::dotenv;
use futures_util::StreamExt;
use rdkafka::{ClientConfig, Message};
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app_config = Settings::new();

    let mut kafka_config: ClientConfig = ClientConfig::new();
    kafka_config.set("bootstrap.servers", app_config.kafka.broker);
    kafka_config.set("group.id", app_config.kafka.consumer_group);
    ;
    // put here to show that the microservice has started
    // println!("Datetime:{},Started...", Local::now());

    let consumer: StreamConsumer = kafka_config.create().unwrap();

    consumer.subscribe(&[&*app_config.kafka.topic]).unwrap();

    let borrowed_msg = consumer.stream().next().await.unwrap().expect("MessageStream never returns None");

    let payload = match borrowed_msg.payload_view::<str>() {
        None => "",
        Some(Ok(s)) => s,
        Some(Err(e)) => {
            println!("Error while deserializing message payload: {:?}", e);
            ""
        }
    };

    println!("Datetime:{}, partition: {}, offset: {}, payload: '{}'",
             Local::now(), borrowed_msg.partition(), borrowed_msg.offset(), payload);

    consumer.commit_message(&borrowed_msg, CommitMode::Sync).expect(&format!("Panic: not able to commit offset:{}", borrowed_msg.offset()));

    // println!("Datetime:{}, Finnished!", Local::now(),);
    Ok(())

    // }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GreetingMessage {
    id: String,
    to: String,
    from: String,
    heading: String,
    message: String,
    created: NaiveDateTime,
}

#[derive(Deserialize)]
pub(crate) struct Settings {
    pub(crate) kafka: Kafka,
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
    pub(crate) message_timeout_ms: i32,
    // pub (crate) enable_idempotence: bool,
    // pub (crate) processing_guarantee: String,
    // pub (crate) number_of_consumers:i32
}
