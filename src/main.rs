use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::net::TcpListener;
use tokio::stream::{Stream, StreamExt};
use tokio::sync::mpsc;
use tokio::time::{DelayQueue, Duration};
use tokio_util::codec::Framed;
use zebra::bgp::*;

struct Peer {
    tx: mpsc::UnboundedSender<Event>,
}

type Shared = HashMap<IpAddr, Peer>;

struct Listener {
    listener: TcpListener,
    rx: mpsc::UnboundedReceiver<IpAddr>,
    timer: DelayQueue<SocketAddr>,
}

impl Stream for Listener {
    type Item = Result<Event, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Poll::Ready(Some(v)) = Pin::new(&mut self.rx).poll_next(cx) {
            let sock = SocketAddr::new(v, BGP_PORT);
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

async fn connect(saddr: SocketAddr, mut rx: mpsc::UnboundedReceiver<Event>) {
    let stream = loop {
        let mut timer = DelayQueue::new();
        timer.insert(Event::TimerExpired, Duration::from_secs(3));

        tokio::select! {
            _ = timer.next() => {
                println!("Start timer expired");
            },
            _ = rx.next() => {
                println!("Stop");
                return;
            },
        };

        timer.insert(Event::TimerExpired, Duration::from_secs(60));
        tokio::select! {
            v = timer.next() => {
                println!("Connect timer expired {:?}", v);
                continue;
            },
            _ = rx.next() => {
                println!("Stop");
                return;
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
        }
    };

    // Framed.
    let mut stream = Framed::new(stream, Bgp {});
    while let Some(x) = stream.next().await {
        println!("{:?}", x);
    }
}

async fn accept(mut streams: Listener, shared: Arc<Mutex<Shared>>) {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Event channel.
    let (tx, rx) = mpsc::unbounded_channel::<IpAddr>();
    let listener = TcpListener::bind(("::", BGP_PORT)).await.unwrap();

    // Listener.
    let streams = Listener {
        listener: listener,
        rx: rx,
        timer: DelayQueue::new(),
    };

    // Test neighbor create.
    let neighbor_addr = IpAddr::V4("192.168.55.2".parse().unwrap());
    tx.send(neighbor_addr).unwrap();

    // Shared status.
    let s = Shared::new();
    let shared: Arc<Mutex<Shared>> = Arc::new(Mutex::new(s));

    futures::join!(accept(streams, shared));

    Ok(())
}
