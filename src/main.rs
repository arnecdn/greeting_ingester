mod my_consumer;

use chrono::NaiveDateTime;
use my_consumer::MyConsumer;

fn main() {
    let app_config = Settings::new();

    let mut consumer = MyConsumer::new( vec![  app_config.kafka.broker ], app_config.kafka.topic , app_config.kafka.consumer_group);

    // put here to show that the microservice has started
    println!("Started...");

    loop {
        for ms in consumer.consume_events().iter() {
            for m in ms.messages() {

                // when the consumer receives an event, this block is executed
                let event_data = MyConsumer::get_event_data(m);
                println!("{}",&event_data.to_string());

            }
            consumer.consume_messageset(ms);
        }
        consumer.commit_consumed();
    }
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

use config::Config;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub (crate) struct Settings {
    pub (crate) kafka: Kafka,
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
pub (crate) struct Kafka {
    pub (crate) broker: String,
    pub (crate) topic: String,
    pub (crate) consumer_group: String,
    pub (crate) message_timeout_ms: i32,
    // pub (crate) enable_idempotence: bool,
    // pub (crate) processing_guarantee: String,
    // pub (crate) number_of_consumers:i32
}
