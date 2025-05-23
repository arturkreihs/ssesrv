use ssesrv::Server;

#[tokio::main]
async fn main() {
    let sse = Server::new();

    // Spawn the warp server
    {
        let sse = sse.clone();
        tokio::spawn(async move {
            sse.run(3030).await;
        });
    }

    // Example: emit messages every 5 seconds
    loop {
        sse.emit("test", "Hello SSE clients!").await;
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

