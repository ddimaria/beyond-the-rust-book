use anyhow::Result;
use std::time::Duration;
use tokio::sync::broadcast::{self, Sender};
use tokio::time::sleep;

#[derive(Clone, Debug)]
enum Message {
    Text(String),
    Shutdown,
}

#[tokio::main]
async fn main() -> Result<()> {
    let (sender, _) = broadcast::channel::<Message>(16);
    let num_subscribers: usize = 10;

    // spawn all subscribers
    for i in 1..=num_subscribers {
        tokio::spawn(new_subscriber(sender.clone(), i));
    }

    // wait for all subscribers to come onlone
    loop {
        if sender.receiver_count() == num_subscribers {
            break;
        }
    }

    // broadcast a message to all subscribers
    sender.send(Message::Text("Hello".into()))?;

    // wait a second
    sleep(Duration::from_millis(1000)).await;

    // tell all subscribers to gracefully shutdown
    sender.send(Message::Shutdown)?;

    // wait for all subscribers to gracefully shutdown
    loop {
        if sender.receiver_count() == 0 {
            break;
        }
    }

    Ok(())
}

async fn new_subscriber(sender: Sender<Message>, label: usize) {
    println!("Spawning Receiver {}", label);

    // subscribe to the sender
    let mut receiver = sender.subscribe();

    // receive all incoming messages
    while let Ok(message) = receiver.recv().await {
        match message {
            Message::Text(text) => println!("Receiver {} received text: {:?}", label, text),
            Message::Shutdown => {
                println!("Receiver {} received shutdown", label);

                // emulate a graceful shutdown
                sleep(Duration::from_millis(1000)).await;

                break;
            }
        }
    }
}
