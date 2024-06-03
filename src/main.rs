use std::time::Duration;

use chrono::NaiveDateTime;
use config::Config;
use dotenv::dotenv;
use rdkafka::{ClientConfig, Message};
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::error::KafkaResult;
use serde::{Deserialize, Serialize};

fn main() {
    let app_config = Settings::new();

    // let mut consumer = MyConsumer::new( vec![  app_config.kafka.broker ], app_config.kafka.topic , app_config.kafka.consumer_group);
    let mut kafka_config: ClientConfig = ClientConfig::new();
    kafka_config.set("bootstrap.servers", app_config.kafka.broker);
    kafka_config.set("group.id", app_config.kafka.consumer_group);
    // kafka_config.set("max.poll.records", "1");
    ;
    // put here to show that the microservice has started
    println!("Started...");

    let consumer: BaseConsumer = kafka_config.create().unwrap();

    consumer.subscribe(&[&*app_config.kafka.topic]).unwrap();
    for msg in consumer.iter() {
        match msg {
            Err(e) => println!("error retrieving msg: {}", e),
            Ok(m) => {
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        println!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };
                println!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                         m.key().unwrap(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
            }
        }


    }
    // match consumer.poll(Duration::from_secs(10)) {
    //     Some(Ok(v)) =>{
    //                     let m = match v.payload_view::<str>() {
    //                         None => "",
    //                         Some(Ok(s)) => s,
    //                         Some(Err(e)) => {
    //                             println!("Error while deserializing message payload: {:?}", e);
    //                             ""
    //                         }
    //                     };
    //         println!("message {:?}", m);
    //     },
    //     Some(Err(e)) => println!("error retrieving msg: {}", e),
    //     _=> (),
    // }


    // Ok(())
    println!("Finnished!");
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
