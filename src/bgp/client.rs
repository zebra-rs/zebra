use crate::bgp::packet::{BgpHeaderPacket, BgpOpenOptPacket, BgpOpenPacket, BgpTypes};
use crate::bgp::{Capabilities, Capability, Family, AFI_IP, BGP_HEADER_LEN, SAFI_MPLS_VPN};
use bytes::BytesMut;
use pnet::packet::Packet;
use std::io::{Error, ErrorKind};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio_util::codec::{Decoder, Encoder};

pub struct Client {
    stream: TcpStream,
}

#[derive(Debug)]
pub enum Event {
    Accept((TcpStream, SocketAddr)),
    Connect(SocketAddr),
    TimerExpired,
    Packet(Message),
}

#[derive(Debug)]
pub enum Message {
    Open(MessageOpen),
    RouteRefresh,
}

impl Message {
    pub fn len(&self) -> usize {
        match self {
            Message::Open(m) => m.len(),
            Message::RouteRefresh => 0,
        }
    }

    pub fn to_bytes(self) -> Result<Vec<u8>, failure::Error> {
        println!("XXX to_bytes");
        let buf: Vec<u8> = Vec::new();
        let mut c = std::io::Cursor::new(buf);
        std::io::Write::write(&mut c, &vec![0xff; 16])?;
        //c.write_u8(10u8)?;
        let vec = c.into_inner();
        println!("XXX to_bytes vec.len {}", vec.len());

        match self {
            Message::RouteRefresh => {
                println!("RouteRefresh");
            }
            _ => {
                println!("Other");
            }
        }

        Ok(vec)
    }
}

#[derive(Debug)]
pub struct MessageOpen {
    len: u16,
    version: u8,
    asn: u16,
    hold_time: u16,
    router_id: Ipv4Addr,
    caps: Capabilities,
}

impl MessageOpen {
    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn from_bytes(buf: &[u8], len: u16) -> Result<Self, failure::Error> {
        let open = BgpOpenPacket::new(buf).ok_or(Error::from(ErrorKind::UnexpectedEof))?;
        let opt_param_len = open.get_opt_param_len() as usize;
        if opt_param_len < open.payload().len() {
            return Err(Error::from(ErrorKind::UnexpectedEof).into());
        }

        let mut caps = Capabilities::new();

        // Open message.
        if opt_param_len > 0 {
            let opt = BgpOpenOptPacket::new(open.payload())
                .ok_or(Error::from(ErrorKind::UnexpectedEof))?;

            // When Open opt message is not capability(2) return here.
            if opt.get_typ() != 2 {
                return Err(Error::from(ErrorKind::UnexpectedEof).into());
            }
            let mut len = opt.get_length() as usize;
            if len < opt.payload().len() {
                return Err(Error::from(ErrorKind::UnexpectedEof).into());
            }

            // Parse Open capability message.
            let mut c = std::io::Cursor::new(opt.payload());

            while len > 0 {
                let pos = c.position();
                match Capability::from_bytes(&mut c) {
                    Ok(cap) => caps.push(cap),
                    Err(e) => {
                        println!("XXX error {}", e);
                        return Err(e);
                    }
                }
                let diff = (c.position() - pos) as usize;
                if diff > len {
                    return Err(Error::from(ErrorKind::UnexpectedEof).into());
                }
                len -= diff;
            }
            println!("XXX caps {:?}", caps)
        }

        Ok(MessageOpen {
            len: len,
            version: open.get_version(),
            asn: open.get_asn(),
            hold_time: open.get_hold_time(),
            router_id: open.get_router_id(),
            caps: caps,
        })
    }
}

// struct MessageUpdate {}

// struct MessageKeepAlive {}

// struct MessageRouteRefresh {}

// struct MessageNotification {}

impl Client {
    pub fn new(stream: TcpStream, _saddr: SocketAddr) -> Self {
        Client {
            stream: stream,
            //saddr: saddr,
        }
    }

    pub async fn open_send(&mut self) {
        // Prepare BGP buffer with marker.
        let mut buf = [0u8; 4096];
        for i in 0..16 {
            buf[i] = 0xff;
        }
        let mut packet = crate::bgp::packet::MutableBgpHeaderPacket::new(&mut buf[0..19]).unwrap();
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
        //let _ = self.stream.write(buf).await;
    }

    pub async fn keepalive_send(&mut self) {
        // Prepare BGP buffer with marker.
        let mut buf = [0u8; 4096];
        for i in 0..16 {
            buf[i] = 0xff;
        }
        let mut packet = crate::bgp::packet::MutableBgpHeaderPacket::new(&mut buf[0..19]).unwrap();
        packet.set_bgp_type(BgpTypes::KEEPALIVE);
        packet.set_length(19u16);

        // Open length.
        let buf = &buf[..19];
        println!("Write {:?}", buf);
        //let _ = self.stream.write(buf).await;
    }

    pub async fn connect(&mut self) -> Result<(), failure::Error> {
        // Send BGP packet.
        self.open_send().await;

        // Read BGP message.
        loop {
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

            let packet = BgpHeaderPacket::new(&buf).unwrap();
            let typ = packet.get_bgp_type();
            let length = packet.get_length();

            println!("Type {:?}", typ);
            match typ {
                BgpTypes::OPEN => {
                    let msg = MessageOpen::from_bytes(packet.payload(), length)?;
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

            self.keepalive_send().await;
        }
    }
}

pub struct Bgp {}

pub fn from_bytes(buf: &[u8]) -> Result<Event, failure::Error> {
    println!("XXX from_bytes len {}", buf.len());
    let n = buf.len();

    if n < 19 {
        // Need to read more.
        println!("BGP packet length is smaller than minimum length (19).");
        return Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe).into());
    }
    println!("Read num: {}", n);

    let packet = BgpHeaderPacket::new(&buf).unwrap();
    let typ = packet.get_bgp_type();
    let len = packet.get_length();

    println!("Type {:?}", typ);
    println!("Length {:?}", len);

    match typ {
        BgpTypes::OPEN => {
            let msg = MessageOpen::from_bytes(packet.payload(), len)?;
            println!("MessageOpen {:?}", msg);
            return Ok(Event::Packet(Message::Open(msg)));
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
    println!("Length {:?}", len);

    return Ok(Event::TimerExpired);
}

impl Decoder for Bgp {
    type Item = Event;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> std::io::Result<Option<Event>> {
        match from_bytes(src) {
            Ok(Event::Packet(m)) => {
                let _ = src.split_to(m.len());
                Ok(Some(Event::Packet(m)))
            }
            Ok(_) => {
                println!("unexpected Ok");
                Ok(None)
            }
            Err(_) => Ok(None),
        }
    }
}

impl Encoder for Bgp {
    type Item = Message;
    type Error = std::io::Error;

    fn encode(&mut self, msg: Message, _dst: &mut BytesMut) -> Result<(), std::io::Error> {
        match msg {
            Message::RouteRefresh => {
                let buf = msg.to_bytes().unwrap();
                println!("XXX RouteRefresh {:?}", buf);
            }
            Message::Open(_) => {
                println!("XXX Open Message encode");
            }
        }
        println!("XXX encode");
        Ok(())
    }
}

use crate::bgp::packet::MutableBgpHeaderPacket;
use crate::bgp::packet::MutableBgpOpenPacket;
use byteorder::WriteBytesExt;

pub fn w(buf: &mut [u8]) -> usize {
    let mut c = std::io::Cursor::new(buf);
    let sp = c.position();
    println!("c position {}", sp);
    let tmp = vec![0xff, 0xff];
    std::io::Write::write(&mut c, &tmp).unwrap();
    println!("c position {}", c.position());
    c.write_u8(10u8).unwrap();
    let cp = c.position();
    let len = cp - sp;
    println!("XXX len {}", len);
    len as usize
}

pub struct PeerConfig {
    asn: u32,
    hold_time: u16,
}

pub fn open_option_packet(_caps: &Capabilities, buf: &mut [u8]) -> usize {
    let c = std::io::Cursor::new(buf);

    c.position() as usize
}

pub fn open_packet(config: &PeerConfig, caps: &Capabilities, buf: &mut [u8]) -> usize {
    // Open.
    let mut len = MutableBgpOpenPacket::minimum_packet_size();
    let mut open = MutableBgpOpenPacket::new(buf).unwrap();
    open.set_version(4);
    open.set_asn(config.asn as u16);
    open.set_hold_time(config.hold_time);
    let router_id: std::net::Ipv4Addr = "10.0.0.1".parse().unwrap();
    open.set_router_id(router_id);

    // Open option.
    len += open_option_packet(caps, &mut buf[len..]);

    len
}

pub fn packet() {
    let mut buf = [0u8; 4096];
    let mut len: usize = BGP_HEADER_LEN;

    // Open Option - capability.
    let mut caps = Capabilities::new();
    let cap_afi = Capability::MultiProtocol(Family {
        afi: AFI_IP,
        safi: SAFI_MPLS_VPN,
    });
    caps.push(cap_afi);

    // Open.
    let config = PeerConfig {
        asn: 1,
        hold_time: 90,
    };
    len += open_packet(&config, &caps, &mut buf[len..]);
    println!("len {}", len);

    // Cursor.
    len += w(&mut buf[len..]);
    println!("len {}", len);

    // Slice.
    let slice = &buf[0..len];
    println!("slice {:?}", slice);

    // Header.
    for i in 0..16 {
        buf[i] = 0xff;
    }
    let mut header = MutableBgpHeaderPacket::new(&mut buf[..len]).unwrap();
    header.set_bgp_type(BgpTypes::OPEN);
    header.set_length(len as u16);
}
