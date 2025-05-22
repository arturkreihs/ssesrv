use std::sync::Arc;
use ssesrv::Server;

#[tokio::main]
async fn main() {
    let server = Arc::new(Server::new());

    // Clone server for the HTTP server task
    let server_for_http = server.clone();

    // Spawn the warp server
    tokio::spawn(async move {
        server_for_http.run(3030).await;
    });

    // Example: emit messages every 5 seconds
    loop {
        server.emit("Hello SSE clients!").await;
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

