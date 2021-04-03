#![allow(unused_imports)]
use futures::sink::SinkExt;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
// use tokio::time::{DelayQueue, Duration};
use tokio::time::Duration;
use tokio_stream::{Stream, StreamExt};
use tokio_util::codec::Framed;
use zebra::bgp::*;

// use log::info;
use slog::{o, slog_info, Drain, Logger};
use slog_term;

struct PeerConfig {
    _tx: mpsc::UnboundedSender<Event>,
}

type Shared = HashMap<IpAddr, PeerConfig>;

struct Listener {
    listener: TcpListener,
    rx: mpsc::UnboundedReceiver<IpAddr>,
    // timer: DelayQueue<SocketAddr>,
}

// impl Stream for Listener {
//     type Item = Result<Event, std::io::Error>;

//     fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//         if let Poll::Ready(Some(v)) = Pin::new(&mut self.rx).poll_next(cx) {
//             let sock = SocketAddr::new(v, BGP_PORT);
//             self.timer.insert(sock, Duration::from_secs(0));
//         }

//         if let Poll::Ready(Some(Ok(v))) = Pin::new(&mut self.timer).poll_expired(cx) {
//             return Poll::Ready(Some(Ok(Event::Connect(v.into_inner()))));
//         }

//         match Pin::new(&mut self.listener).poll_accept(cx) {
//             Poll::Ready(Ok((socket, addr))) => Poll::Ready(Some(Ok(Event::Accept((socket, addr))))),
//             Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e))),
//             Poll::Pending => Poll::Pending,
//         }
//     }
// }

async fn connect(saddr: SocketAddr, rx: mpsc::UnboundedReceiver<Event>) {
    let stream = loop {
        // let mut timer = DelayQueue::new();
        // timer.insert(Event::TimerExpired, Duration::from_secs(3));

        // tokio::select! {
        // _ = timer.next() => {
        //     println!("Start timer expired");
        // },
        // _ = rx.next() => {
        //     println!("Stop");
        //     return;
        // },
        // };

        // let mut timer = DelayQueue::new();
        // timer.insert(Event::TimerExpired, Duration::from_secs(60));
        tokio::select! {
            // v = timer.next() => {
            //     println!("Connect timer expired {:?}", v);
            //     continue;
            // },
            // _ = rx.next() => {
            //     println!("Stop");
            //     return;
            // },
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
    let peer = Peer {
        state: State::OpenSent,
    };
    let mut stream = Framed::new(stream, peer);

    // Send open.
    let msg = Message::Open(MessageOpen::new());
    if stream.send(msg).await.is_err() {
        println!("OpenMessage send error");
    } else {
        println!("OpenMessage send success");
    }

    let mut state = State::OpenSent;

    // let mut timer = DelayQueue::new();
    // timer.insert(Event::TimerExpired, Duration::from_secs(60));

    while let Some(x) = stream.next().await {
        match x {
            Ok(Message::None) => {
                break;
            }
            Ok(Message::Open(_)) => {
                println!("Got Open message");
                match state {
                    State::OpenSent => {
                        println!("OpenSent");
                        if stream.send(Message::KeepAlive).await.is_err() {
                            println!("Keepalive send error");
                        } else {
                            println!("Keepalive send success");
                        }
                        state = State::OpenSent;
                    }
                    _ => {
                        println!("OpenSent");
                    }
                }
            }
            Ok(Message::KeepAlive) => {
                println!("Got KeepAlive message");
            }
            y => {
                println!("XXXX other messages {:?}", y);
            }
        }
        println!("stream await returns");
    }
    println!("XXX while stream.next() end");
}

// async fn accept(mut streams: Listener, shared: Arc<Mutex<Shared>>) {
//     loop {
//         let (_stream, _saddr) = match streams.next().await {
//             Some(v) => match v {
//                 Ok(Event::Accept((stream, saddr))) => {
//                     println!("Accept!");
//                     (stream, saddr)
//                 }
//                 Ok(Event::Connect(saddr)) => {
//                     println!("New peer event {}", saddr);

//                     let (tx, rx) = mpsc::unbounded_channel::<Event>();

//                     // // Main task.
//                     {
//                         let a = IpAddr::V4("192.168.55.2".parse().unwrap());
//                         let p = PeerConfig { _tx: tx };
//                         let mut peers = shared.lock().unwrap();
//                         peers.insert(a, p);
//                     }
//                     let _shared = Arc::clone(&shared);

//                     tokio::spawn(connect(saddr, rx));
//                     continue;
//                 }
//                 Ok(_) => {
//                     continue;
//                 }
//                 Err(e) => {
//                     println!("Error: {}", e);
//                     continue;
//                 }
//             },
//             None => {
//                 continue;
//             }
//         };
//         // println!("{:?} {}", stream, saddr);
//         // {
//         //     let peers = shared.lock().unwrap();
//         //     println!("----");
//         //     for (k, _) in peers.iter() {
//         //         println!("{:?}", k);
//         //     }
//         //     println!("----");
//         // }

//         // tokio::spawn(async move {
//         //     let _ = bgp::Client::new(stream, saddr).connect().await;
//         // });
//     }
// }

#[tokio::main]
pub async fn main() -> Result<()> {
    // let plain = slog_term::PlainSyncDecorator::new(std::io::stdout());
    // let logger = Logger::root(slog_term::FullFormat::new(plain).build().fuse(), o!());

    // slog_info!(logger, "zebra bgpd started");

    // // Event channel.
    // let (tx, rx) = mpsc::unbounded_channel::<IpAddr>();
    // let listener = TcpListener::bind(("::", BGP_PORT)).await.unwrap();

    // // Listener.
    // let streams = Listener {
    //     listener: listener,
    //     rx: rx,
    //     // timer: DelayQueue::new(),
    // };

    // // Test neighbor create.
    // let neighbor_addr = IpAddr::V4("192.168.55.2".parse().unwrap());
    // tx.send(neighbor_addr).unwrap();

    // // Shared status.
    // let s = Shared::new();
    // let shared: Arc<Mutex<Shared>> = Arc::new(Mutex::new(s));

    // futures::join!(accept(streams, shared));

    // Ok(())
}
