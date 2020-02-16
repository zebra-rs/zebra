#![allow(dead_code)]
//use super::*;
use crate::bgp::packet::{BgpOpenOptPacket, BgpOpenPacket, BgpPacket, BgpTypes};
use crate::bgp::{capability_parse, Capabilities};
use pnet::packet::Packet;
use std::io::{Error, ErrorKind};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct Client {
    stream: TcpStream,
    saddr: SocketAddr,
}

enum Message {
    MessageOpen,
    MessageUpdate,
    MessageKeepAlive,
    MessageNotification,
    MessageRouteRefresh,
}

#[derive(Debug)]
struct MessageOpen {
    version: u8,
    asn: u16,
    hold_time: u16,
    router_id: Ipv4Addr,
    caps: Capabilities,
}

impl MessageOpen {
    pub fn from_bytes(buf: &[u8]) -> Result<Self, std::io::Error> {
        let open = BgpOpenPacket::new(buf).ok_or(Error::from(ErrorKind::UnexpectedEof))?;
        let opt_param_len = open.get_opt_param_len() as usize;
        if opt_param_len < open.payload().len() {
            return Err(Error::from(ErrorKind::UnexpectedEof));
        }

        let mut caps = Capabilities::new();

        // Open message.
        if opt_param_len > 0 {
            let opt = BgpOpenOptPacket::new(open.payload())
                .ok_or(Error::from(ErrorKind::UnexpectedEof))?;

            // When Open opt message is not capability(2) return here.
            if opt.get_typ() != 2 {
                return Err(Error::from(ErrorKind::UnexpectedEof));
            }
            let opt_len = opt.get_length() as usize;
            if opt_len < opt.payload().len() {
                return Err(Error::from(ErrorKind::UnexpectedEof));
            }

            // Parse Open capability message.
            let mut buf: &[u8] = opt.payload();
            while let Some(payload) = capability_parse(buf, &mut caps) {
                buf = payload;
                println!("len {}", buf.len());
            }
            caps.dump();
        }

        Ok(MessageOpen {
            version: open.get_version(),
            asn: open.get_asn(),
            hold_time: open.get_hold_time(),
            router_id: open.get_router_id(),
            caps: caps,
        })
    }
}

struct MessageUpdate {}

struct MessageKeepAlive {}

struct MessageRouteRefresh {}

struct MessageNotification {}

impl Client {
    pub fn new(stream: TcpStream, saddr: SocketAddr) -> Self {
        Client {
            stream: stream,
            saddr: saddr,
        }
    }

    pub fn open_send(&mut self) {
        // Prepare BGP buffer with marker.
        let mut buf = [0u8; 4096];
        for i in 0..16 {
            buf[i] = 0xff;
        }
        let mut packet = crate::bgp::packet::MutableBgpPacket::new(&mut buf[0..19]).unwrap();
        packet.set_bgp_type(BgpTypes::OPEN);
        packet.set_length(29u16);

        let mut open = crate::bgp::packet::MutableBgpOpenPacket::new(&mut buf[19..]).unwrap();
        open.set_version(4);
        open.set_asn(1);
        open.set_hold_time(3);
        let router_id: std::net::Ipv4Addr = "10.0.0.1".parse().unwrap();
        open.set_router_id(router_id);

        // Open length.
        let buf = &buf[..29];
        println!("Write {:?}", buf);
        let _ = self.stream.write(buf);
    }

    pub async fn connect(&mut self) -> Result<(), failure::Error> {
        // Send BGP packet.
        self.open_send();

        // Read BGP message.
        let mut buf = [0u8; 4096];
        let n = self.stream.read(&mut buf).await?;
        if n == 0 {
            println!("BGP socket closed");
            std::process::exit(1);
        }
        let buf = &buf[0..n];
        println!("Read {:?}", buf);

        // Minimum BGP message len is 19.
        if n < 19 {
            // Need to read more.
            println!("BGP packet length is smaller than minimum length (19).");
            std::process::exit(1);
        }
        println!("Read num: {}", n);

        let packet = BgpPacket::new(&buf).unwrap();
        let typ = packet.get_bgp_type();
        let length = packet.get_length();

        println!("Type {:?}", typ);
        match typ {
            BgpTypes::OPEN => {
                let msg = MessageOpen::from_bytes(packet.payload())?;
                println!("MessageOpen {:?}", msg);
            }
            BgpTypes::UPDATE => {
                println!("Update message!");
            }
            BgpTypes::NOTIFICATION => {
                println!("Notification message!");
            }
            BgpTypes::KEEPALIVE => {
                println!("Keepalive message!");
            }
            unknown => {
                println!("Unknown message type {:?}", unknown);
            }
        }

        println!("Length {:?}", length);

        // loop {}

        Ok(())
    }
}
