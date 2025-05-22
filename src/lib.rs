// Cargo.toml dependencies:
// [dependencies]
// warp = "0.3"
// tokio = { version = "1", features = ["full"] }
// futures-util = "0.3"

use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use warp::sse::Event;
use warp::Filter;
use futures_util::stream::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

type Clients = Arc<Mutex<Vec<mpsc::UnboundedSender<Event>>>>;

pub struct Server {
    clients: Clients,
}

impl Server {
    pub fn new() -> Self {
        Server {
            clients: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Starts the warp server on port 3030 with /events SSE endpoint.
    pub async fn run(self: Arc<Server>) {
        let clients_filter = warp::any().map(move || self.clients.clone());

        let events_route = warp::path("events")
            .and(warp::get())
            .and(clients_filter)
            .map(|clients: Clients| {
                // Create a channel for sending SSE events to this client
                let (tx, rx) = mpsc::unbounded_channel();

                // Register this sender so we can broadcast to it later
                tokio::spawn(async move {
                    clients.lock().await.push(tx);
                });

                // Convert receiver into a stream of Result<Event, Infallible>
                let event_stream = UnboundedReceiverStream::new(rx)
                    .map(|event| Ok(event) as Result<Event, Infallible>);

                // Reply with SSE stream and keep-alive
                warp::sse::reply(warp::sse::keep_alive().stream(event_stream))
            });

        println!("Server running on http://0.0.0.0:3030/events");
        warp::serve(events_route).run(([0, 0, 0, 0], 3030)).await;
    }

    /// Broadcast a message to all connected clients.
    pub async fn emit(&self, msg: &str) {
        let mut clients = self.clients.lock().await;

        // Retain only clients that successfully receive the message
        clients.retain(|client| {
            let event = Event::default().data(msg.to_string());
            client.send(event).is_ok()
        });
    }
}
