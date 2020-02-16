use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::{Stream, StreamExt};
use tokio::sync::mpsc;
use tokio::time::{DelayQueue, Duration};
use zebra::bgp;

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
            let sock = std::net::SocketAddr::new(v, bgp::BGP_PORT);
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
    // Event channel.
    let (tx, rx) = mpsc::unbounded_channel::<IpAddr>();
    let listener = TcpListener::bind(("::", bgp::BGP_PORT)).await.unwrap();

    // Streams.
    let mut streams = Streams {
        listener: listener,
        rx: rx,
        timer: DelayQueue::new(),
    };

    // Test neighbor create.
    let neighbor_addr = IpAddr::V4("192.168.55.2".parse()?);
    tx.send(neighbor_addr)?;

    loop {
        let (stream, saddr) = match streams.next().await {
            Some(v) => match v {
                Ok(Event::Accept((stream, saddr))) => {
                    println!("Accept");
                    (stream, saddr)
                }
                Ok(Event::Connect(saddr)) => {
                    println!("Trying to Connect {}", saddr);
                    match TcpStream::connect(saddr).await {
                        Ok(stream) => {
                            println!("Connect success");
                            (stream, saddr)
                        }
                        Err(_) => {
                            println!("connect error");
                            streams.timer.insert(saddr, Duration::from_secs(15));
                            continue;
                        }
                    }
                }
                Err(e) => {
                    println!("Error: {}", e);
                    continue;
                }
            },
            None => {
                continue;
            }
        };
        println!("Here we are {:?} {}", stream, saddr);
        tokio::spawn(async move {
            let mut client = bgp::Client::new(stream, saddr);
            client.connect().unwrap();
        });
    }
}
