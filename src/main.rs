#![allow(dead_code)]

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::{Stream, StreamExt};
use tokio::sync::mpsc;
use tokio::time::{DelayQueue, Duration};
//use tokio_util::codec::Framed;
use zebra::bgp;

struct PreSession {}

struct Peer {
    tx: mpsc::UnboundedSender<IpAddr>,
}

type Shared = HashMap<IpAddr, Peer>;

enum Event {
    Accept((TcpStream, SocketAddr)),
    Connect(SocketAddr),
}

struct Listener {
    listener: TcpListener,
    rx: mpsc::UnboundedReceiver<IpAddr>,
    timer: DelayQueue<SocketAddr>,
}

impl Stream for Listener {
    type Item = Result<Event, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Poll::Ready(Some(v)) = Pin::new(&mut self.rx).poll_next(cx) {
            let sock = SocketAddr::new(v, bgp::BGP_PORT);
            self.timer.insert(sock, Duration::from_secs(0));
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

struct Connector {
    rx: mpsc::UnboundedReceiver<IpAddr>,
    timer: DelayQueue<i32>,
}

impl Stream for Connector {
    type Item = Result<Event, std::io::Error>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // if let Some(_) = Pin::new(&mut self.conn) {
        //     println!("XXX Pin conn stream");
        // }
        Poll::Pending
    }
}

// struct Session {
//     frames: Framed<TcpStream, Bgp>,
// }

async fn connect(saddr: SocketAddr, mut _sess: Connector) {
    loop {
        println!("XXX Trying to connect");
        let stream = match TcpStream::connect(saddr).await {
            Ok(stream) => stream,
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        };
        println!("XXX connected {:?}", stream);

        // Need to check we already have established session or not.

        // When session is already established, close stream then fell in sleep.

        break;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Event channel.
    let (tx, rx) = mpsc::unbounded_channel::<IpAddr>();
    let listener = TcpListener::bind(("::", bgp::BGP_PORT)).await.unwrap();

    // Listener.
    let mut streams = Listener {
        listener: listener,
        rx: rx,
        timer: DelayQueue::new(),
    };

    // Test neighbor create.
    //let neighbor_addr = IpAddr::V4("192.168.55.2".parse()?);
    let neighbor_addr = IpAddr::V4("10.0.0.3".parse()?);
    tx.send(neighbor_addr)?;

    // Shared status.
    let s = Shared::new();
    let shared: Arc<Mutex<Shared>> = Arc::new(Mutex::new(s));

    loop {
        let (stream, saddr) = match streams.next().await {
            Some(v) => match v {
                Ok(Event::Accept((stream, saddr))) => {
                    println!("Accept!");
                    (stream, saddr)
                }
                Ok(Event::Connect(saddr)) => {
                    println!("New peer event {}", saddr);

                    let (tx, rx) = mpsc::unbounded_channel::<IpAddr>();

                    // Main task.
                    {
                        let a = IpAddr::V4("192.168.55.2".parse().unwrap());
                        let p = Peer { tx };
                        let mut peers = shared.lock().unwrap();
                        peers.insert(a, p);
                    }

                    let _shared = Arc::clone(&shared);
                    let sess = Connector {
                        rx,
                        timer: DelayQueue::new(),
                    };

                    // We've got connect event.
                    tokio::spawn(connect(saddr, sess));

                    continue;

                    // match TcpStream::connect(saddr).await {
                    //     Ok(stream) => {
                    //         println!("Connect!");
                    //         (stream, saddr)
                    //     }
                    //     Err(e) => {
                    //         println!("Connect error: {}", e);
                    //         streams.timer.insert(saddr, Duration::from_secs(15));
                    //         continue;
                    //     }
                    // }
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
        println!("{:?} {}", stream, saddr);
        {
            let peers = shared.lock().unwrap();
            println!("----");
            for (k, _) in peers.iter() {
                println!("{:?}", k);
            }
            println!("----");
        }

        tokio::spawn(async move {
            let _ = bgp::Client::new(stream, saddr).connect().await;
        });
    }
}
