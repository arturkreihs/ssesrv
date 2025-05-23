use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use warp::{Filter, sse::Event};
use futures_util::stream::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

type Clients = Arc<Mutex<Vec<mpsc::UnboundedSender<Event>>>>;

#[derive(Debug, Clone)]
pub struct Server {
    clients: Clients,
}

impl Server {
    pub fn new() -> Self {
        Server {
            clients: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn run(self: Server, port: u16) {
        let filter = warp::any().map(move || self.clients.clone());
        let route = warp::path("events")
            .and(warp::get())
            .and(filter)
            .map(|clients: Clients| {
                let (tx, rx) = mpsc::unbounded_channel();
                tokio::spawn(async move {
                    clients.lock().await.push(tx);
                });
                let event_stream = UnboundedReceiverStream::new(rx)
                    .map(|event| Ok(event) as Result<Event, Infallible>);
                let stream = warp::sse::keep_alive().stream(event_stream);
                warp::sse::reply(stream)
            });
        warp::serve(route).run(([0, 0, 0, 0], port)).await;
    }

    pub async fn emit(&self, msg: &str) {
        let mut clients = self.clients.lock().await;
        clients.retain(|client| {
            let event = Event::default().data(msg.to_owned());
            client.send(event).is_ok()
        });
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}
