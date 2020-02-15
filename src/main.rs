use std::net::{IpAddr, Ipv4Addr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
//use tokio::runtime::Builder;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

struct Streams {
    listener: TcpListener,
    rx: mpsc::UnboundedReceiver<IpAddr>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Please specify BGP neighbor IP address");
        std::process::exit(1);
    }

    //let mut rt = Builder::new().basic_scheduler().enable_all().build()?;
    let mut rt = Runtime::new()?;

    let (tx, rx) = mpsc::unbounded_channel::<IpAddr>();

    // Test send.
    let neighbor_ipv4: Ipv4Addr = "10.0.0.1".parse()?;
    let neighbor_addr = IpAddr::V4(neighbor_ipv4);
    tx.send(neighbor_addr)?;

    rt.block_on(async move {
        tokio::spawn(async move {
            let client = zebra::bgp::Client::new(&args[1]);
            if let Err(err) = client.connect() {
                println!("{}", err);
                //std::process::exit(1);
                println!("client connect failed");
            } else {
                println!("client connect success");
            }
        });

        let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();

        // Streams.
        let mut stream = Streams {
            listener: listener,
            rx: rx,
        };

        let addr = stream.rx.recv().await;
        println!("recv {:?}", addr);

        loop {
            let (mut socket, _) = stream.listener.accept().await.unwrap();
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
    });

    Ok(())
}
