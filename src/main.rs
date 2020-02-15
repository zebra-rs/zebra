use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::pin::Pin;
use std::task::{Context, Poll};
//use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::{Stream, StreamExt};
use tokio::sync::mpsc;
use tokio::time::{DelayQueue, Duration};

static BGP_PORT: u16 = 179;

struct Streams {
    listener: TcpListener,
    rx: mpsc::UnboundedReceiver<IpAddr>,
    timer: DelayQueue<SocketAddr>,
}

enum Event {
    Accept((TcpStream, SocketAddr)),
    Connect(SocketAddr),
}

impl Stream for Streams {
    type Item = Result<Event, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Poll::Ready(Some(v)) = Pin::new(&mut self.rx).poll_next(cx) {
            let sock = std::net::SocketAddr::new(v, BGP_PORT);
            self.timer.insert(sock, Duration::from_secs(5));
        }

        if let Poll::Ready(Some(Ok(v))) = Pin::new(&mut self.timer).poll_expired(cx) {
            return Poll::Ready(Some(Ok(Event::Connect(v.into_inner()))));
        }

        match Pin::new(&mut self.listener).poll_accept(cx) {
            Poll::Ready(Ok((socket, addr))) => Poll::Ready(Some(Ok(Event::Accept((socket, addr))))),
            Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e))),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Please specify BGP neighbor IP address");
        std::process::exit(1);
    }

    // Test client.
    // tokio::spawn(async move {
    //     let client = zebra::bgp::Client::new(&args[1]);
    //     if let Err(err) = client.connect() {
    //         println!("{}", err);
    //         //std::process::exit(1);
    //         println!("client connect failed");
    //     } else {
    //         println!("client connect success");
    //     }
    // });

    // Event channel.
    let (tx, rx) = mpsc::unbounded_channel::<IpAddr>();
    let listener = TcpListener::bind(("0.0.0.0", BGP_PORT)).await.unwrap();

    // Streams.
    let mut streams = Streams {
        listener: listener,
        rx: rx,
        timer: DelayQueue::new(),
    };

    // Test send.
    let neighbor_ipv4: Ipv4Addr = "10.0.0.1".parse()?;
    let neighbor_addr = IpAddr::V4(neighbor_ipv4);
    tx.send(neighbor_addr)?;

    loop {
        match streams.next().await {
            Some(v) => match v {
                Ok(Event::Accept((_, _))) => {
                    println!("Accept");
                }
                Ok(Event::Connect(_)) => {
                    println!("Connect");
                }
                Err(e) => println!("Error: {}", e),
            },
            None => {}
        };
        // let (mut socket, _) = stream.listener.accept().await.unwrap();
        // println!("Accept");

        // tokio::spawn(async move {
        //     let mut buf = [0; 4096];

        //     let n = socket.read(&mut buf).await.unwrap();
        //     if n == 0 {
        //         return;
        //     }
        //     socket.write_all(&buf[0..n]).await.unwrap();
        // });
    }
}
