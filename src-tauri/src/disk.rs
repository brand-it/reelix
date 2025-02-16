use crate::services::shell;
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use tokio::net::TcpListener;

pub async fn watch_for_changes() {
    // let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Watching for changes");
    tokio::spawn(async move {
        // let mut buf = [0; 1024];
        println!("spawn");
        // In a loop, read data from the socket and write the data back.
        loop {
            shell::list_disks()
            // let n = match socket.read(&mut buf).await {
            //     // socket closed
            //     Ok(0) => return,
            //     Ok(n) => n,
            //     Err(e) => {
            //         eprintln!("failed to read from socket; err = {:?}", e);
            //         return;
            //     }
            // };

            // // Write the data back
            // if let Err(e) = socket.write_all(&buf[0..n]).await {
            //     eprintln!("failed to write to socket; err = {:?}", e);
            //     return;
            // }
        }
    });
}
