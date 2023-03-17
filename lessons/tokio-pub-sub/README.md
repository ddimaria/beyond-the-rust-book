# Tokio PubSub with Graceful Shutdown

Let's create a simple pubsub service using Tokio's [broadcast](https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html) channels.  We'll be passing messages to the subscribers, so let's define that enum:

```rust
enum Message {
    Text(String),
    Shutdown,
}
```

The sedner can either send text messages or tell the subscriber to shutdown.  Before we can send a message, let's spin up the subscribers:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let (sender, _) = broadcast::channel::<Message>(16);
    let num_subscribers: usize = 10;

    // spawn all subscribers
    for i in 1..=num_subscribers {
        tokio::spawn(new_subscriber(sender.clone(), i));
    }

    // the rest of main
}
```

We will create 10 subscribers and spawn each in a separate thread.  Let's wait for all subscribers to come online using the `receiver_count()` function as a tally of the receivers before we publish any messages:

```rust
loop {
    if sender.receiver_count() == num_subscribers {
        break;
    }
}
```

The full `new_subscriber` function is below, but we'll break down each section shortly:

```rust
async fn new_subscriber(sender: Sender<Message>, label: usize) {
    println!("Spawning Receiver {}", label);

    let mut receiver = sender.subscribe();

    while let Ok(message) = receiver.recv().await {
        match message {
            Message::Text(text) => println!("Receiver {} received text: {:?}", label, text),
            Message::Shutdown => {
                println!("Receiver {} received shutdown", label);
                sleep(Duration::from_millis(1000)).await;
                break;
            }
        }
    }
}
```

To receive broadcast messages, we'll first need to subscibe to them:

```rust
let mut receiver = sender.subscribe();
```

We're now ready to receive messages.  The sender broadcasts a message to all subscribers:

```rust
sender.send(Message::Text("Hello".into()))?;
```
The subscribers simply print the message:

```rust
Message::Text(text) => println!("Receiver {} received text: {:?}", label, text),
```

The sender can tell each subscriber to gracefully shutdown.  We could simply `abort` the `JoinHandle` for each spawned thread, but services in the real world need to perform some shutdown operations first.  

The sender first tells all of the subscribers to shutdown:

```rust
sender.send(Message::Shutdown)?;
```

On the receiver side, we can just wait a second to simulate a shutdown operation then break out of the loop:

```rust
Message::Shutdown => {
    println!("Receiver {} received shutdown", label);

    // emulate a graceful shutdown
    sleep(Duration::from_millis(1000)).await;

    break;
}
```

When we break out of the loop, the receiver goes out of scope and is dropped.  The `receiver_count()` function on the sender reflects this.  We'll wait for all subscribers to shutdown before exiting the program:

```rust
loop {
    if sender.receiver_count() == 0 {
      break;
    }
}
```

We can run the binary and view the output (your output may be different due to timing):

```text
Spawning Receiver 1
Spawning Receiver 3
Spawning Receiver 5
Spawning Receiver 4
Spawning Receiver 7
Spawning Receiver 9
Spawning Receiver 10
Spawning Receiver 8
Spawning Receiver 6
Spawning Receiver 2
Receiver 1 received text: "Hello"
Receiver 3 received text: "Hello"
Receiver 5 received text: "Hello"
Receiver 4 received text: "Hello"
Receiver 8 received text: "Hello"
Receiver 7 received text: "Hello"
Receiver 2 received text: "Hello"
Receiver 10 received text: "Hello"
Receiver 6 received text: "Hello"
Receiver 9 received text: "Hello"
Receiver 5 received shutdown
Receiver 4 received shutdown
Receiver 8 received shutdown
Receiver 6 received shutdown
Receiver 9 received shutdown
Receiver 3 received shutdown
Receiver 2 received shutdown
Receiver 7 received shutdown
Receiver 1 received shutdown
Receiver 10 received shutdown
```

