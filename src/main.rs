use futures::future::join_all;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::runtime::Builder;
//use tokio::time::Duration;

// async fn fff() -> Result<(), Box<dyn std::error::Error>> {
//     tokio::time::delay_for(Duration::from_secs(3)).await;

//     let mut _listener = TcpListener::bind("127.0.0.1:7878").await?;
//     Ok(())
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Please specify BGP neighbor IP address");
        std::process::exit(1);
    }

    let mut rt = Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap();

    //let _ = rt.block_on(fff());

    rt.block_on(async {
        let mut handles = Vec::new();

        handles.push(tokio::spawn(async {
            // Listener.
            let mut listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();
            println!("Start listener");

            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                println!("Accept");

                tokio::spawn(async move {
                    let mut buf = [0; 4096];

                    let n = socket.read(&mut buf).await.unwrap();
                    if n == 0 {
                        return;
                    }
                    socket.write_all(&buf[0..n]).await.unwrap();
                });
            }
        }));

        handles.push(tokio::spawn(async move {
            let client = zebra::bgp::Client::new(&args[1]);
            if let Err(err) = client.connect() {
                println!("{}", err);
                //std::process::exit(1);
                println!("client connect failed");
            } else {
                println!("client connect success");
            }
        }));

        join_all(handles).await;
    });

    Ok(())
}
