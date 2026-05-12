use borsh::{BorshDeserialize, BorshSerialize};
use lapin::{Connection, ConnectionProperties, options::*, types::FieldTable, ExchangeKind};
use futures::stream::StreamExt;

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct UserCreatedEventMessage {
    pub user_id: String,
    pub user_name: String
}

#[tokio::main]
async fn main() {
    let uri = "amqp://guest:guest@localhost:5672";
    let connection = Connection::connect(
        uri,
        ConnectionProperties::default(),
    )
    .await
    .unwrap();

    let channel = connection.create_channel().await.unwrap();

    // Declare the exchange to match publisher
    channel
        .exchange_declare(
            "user_created",
            ExchangeKind::Fanout,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    // Declare the queue
    channel
        .queue_declare(
            "user_created_queue",
            QueueDeclareOptions {
                durable: true,
                auto_delete: false,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .unwrap();

    // Bind the queue to the exchange with empty routing key (for Fanout exchange)
    channel
        .queue_bind(
            "user_created_queue",
            "user_created",
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    // Set up consumer
    let consumer = channel
        .basic_consume(
            "user_created_queue",
            "user_created_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    println!("Waiting for messages on user_created queue...");

    let mut consumer_stream = consumer;
    while let Some(delivery) = consumer_stream.next().await {
        match delivery {
            Ok(delivery) => {
                match UserCreatedEventMessage::try_from_slice(&delivery.data) {
                    Ok(message) => {
                        println!("In Marco's Computer [2406411824]. Message received: {:?}", message);

                        // Simulate slow subscriber
                        //tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

                        delivery.ack(BasicAckOptions::default()).await.unwrap();
                    }
                    Err(e) => {
                        eprintln!("Failed to deserialize message: {:?}", e);
                        delivery.nack(BasicNackOptions::default()).await.unwrap();
                    }
                }
            }
            Err(e) => {
                eprintln!("Consumer error: {:?}", e);
            }
        }
    }
}