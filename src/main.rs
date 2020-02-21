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

struct Peer {
    tx: mpsc::UnboundedSender<Event>,
}

type Shared = HashMap<IpAddr, Peer>;

#[derive(Debug)]
enum Event {
    Accept((TcpStream, SocketAddr)),
    Connect(SocketAddr),
    TimerExpired,
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

use Event::*;

async fn connect(saddr: SocketAddr, mut rx: mpsc::UnboundedReceiver<Event>) {
    let sock = loop {
        let mut timer = DelayQueue::new();
        timer.insert(TimerExpired, Duration::from_secs(3));

        tokio::select! {
            Some(_) = timer.next() => {
                println!("Start timer expired");
            },
            Some(_) = rx.next() => {
                println!("RX");
            },
        };

        tokio::select! {
            _ = timer.next() => {
                println!("XXX timer should not happen");
                continue;
            },
            v = tokio::net::TcpStream::connect(saddr) => {
                match v {
                    Ok(v) => {
                        break v;
                    }
                    Err(e) => {
                        println!("Connect Error {:?}", e);
                        continue;
                    }
                }
            }
            _ = rx.next() => {
                println!("RX");
                continue;
            },
        }
    };
    // SockStream.
    println!("sock {:?}", sock);
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
    let neighbor_addr = IpAddr::V4("192.168.55.2".parse()?);
    tx.send(neighbor_addr)?;

    // Shared status.
    let s = Shared::new();
    let shared: Arc<Mutex<Shared>> = Arc::new(Mutex::new(s));

    loop {
        let (_stream, _saddr) = match streams.next().await {
            Some(v) => match v {
                Ok(Event::Accept((stream, saddr))) => {
                    println!("Accept!");
                    (stream, saddr)
                }
                Ok(Event::Connect(saddr)) => {
                    println!("New peer event {}", saddr);

                    let (tx, rx) = mpsc::unbounded_channel::<Event>();

                    // Main task.
                    {
                        let a = IpAddr::V4("192.168.55.2".parse().unwrap());
                        let p = Peer { tx };
                        let mut peers = shared.lock().unwrap();
                        peers.insert(a, p);
                    }
                    let _shared = Arc::clone(&shared);

                    tokio::spawn(connect(saddr, rx));
                    continue;
                }
                Ok(_) => {
                    continue;
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
        // println!("{:?} {}", stream, saddr);
        // {
        //     let peers = shared.lock().unwrap();
        //     println!("----");
        //     for (k, _) in peers.iter() {
        //         println!("{:?}", k);
        //     }
        //     println!("----");
        // }

        // tokio::spawn(async move {
        //     let _ = bgp::Client::new(stream, saddr).connect().await;
        // });
    }
}
