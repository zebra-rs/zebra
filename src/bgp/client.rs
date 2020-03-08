use crate::bgp::packet::MutableBgpHeaderPacket;
use crate::bgp::packet::MutableBgpOpenPacket;
use crate::bgp::packet::{BgpHeaderPacket, BgpOpenOptPacket, BgpOpenPacket, BgpTypes};
use crate::bgp::{Capabilities, Capability, Family, AFI_IP, BGP_HEADER_LEN, SAFI_MPLS_VPN};
use bytes::BytesMut;
use pnet::packet::Packet;
use std::io::{Error, ErrorKind};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::TcpStream;
use tokio_util::codec::{Decoder, Encoder};

pub struct Client {}

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
    KeepAlive,
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
    pub fn new() -> Self {
        let mut caps = Capabilities::new();
        let cap_afi = Capability::MultiProtocol(Family {
            afi: AFI_IP,
            safi: SAFI_MPLS_VPN,
        });
        caps.push(cap_afi);

        MessageOpen {
            len: 0,
            version: 4,
            asn: 1,
            hold_time: 90,
            router_id: "10.0.0.1".parse().unwrap(),
            caps,
        }
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn from_bytes(buf: &[u8], len: u16) -> Result<Self, failure::Error> {
        let open = BgpOpenPacket::new(buf).ok_or(Error::from(ErrorKind::UnexpectedEof))?;
        let opt_len = open.get_opt_param_len() as usize;
        if opt_len < open.payload().len() {
            return Err(Error::from(ErrorKind::UnexpectedEof).into());
        }

        let mut caps = Capabilities::new();
        if opt_len > 0 {
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

    pub fn to_bytes(&self, buf: &mut [u8]) -> Result<usize, failure::Error> {
        let offset = MutableBgpOpenPacket::minimum_packet_size();
        let len = self.caps.to_bytes(&mut buf[offset..])?;

        let mut open = MutableBgpOpenPacket::new(buf).unwrap();
        open.set_version(self.version);
        open.set_asn(self.asn as u16);
        open.set_hold_time(self.hold_time);
        open.set_router_id(self.router_id);
        open.set_opt_param_len(len as u8);

        Ok(offset + len)
    }
}

pub struct Bgp {}

impl Message {
    // Used by decode.
    pub fn len(&self) -> usize {
        match self {
            Message::Open(m) => m.len(),
            Message::KeepAlive => 0,
        }
    }

    // Called from encode().
    pub fn to_bytes(self) -> Result<Vec<u8>, failure::Error> {
        let mut buf = [0u8; 4096];
        let mut len: usize = BGP_HEADER_LEN;

        match self {
            Message::Open(m) => {
                len += m.to_bytes(&mut buf[len..])?;
            }
            _ => {}
        }

        // BGP Header.
        for i in 0..16 {
            buf[i] = 0xff;
        }
        let mut header = MutableBgpHeaderPacket::new(&mut buf[..len]).unwrap();
        header.set_bgp_type(BgpTypes::OPEN);
        header.set_length(len as u16);

        Ok((&buf[..len]).to_vec())
    }
}

impl Encoder for Bgp {
    type Item = Message;
    type Error = failure::Error;

    fn encode(&mut self, msg: Message, dst: &mut BytesMut) -> Result<(), failure::Error> {
        let buf = msg.to_bytes()?;
        dst.extend_from_slice(&buf);
        Ok(())
    }
}

pub fn from_bytes(buf: &[u8]) -> Result<Message, failure::Error> {
    println!("--------------------");
    println!("RECV: Buffer length {}", buf.len());
    let n = buf.len();

    if n == 0 {
        println!("XXX read length is zero");
        std::process::exit(1);
    }

    if n < 19 {
        // Need to read more.
        println!("BGP packet length is smaller than minimum length (19).");
        return Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe).into());
    }
    let packet = BgpHeaderPacket::new(&buf).unwrap();
    let typ = packet.get_bgp_type();
    let len = packet.get_length();

    println!("RECV: Header Type {:?}", typ);
    println!("RECV: Header length {:?}", len);

    match typ {
        BgpTypes::OPEN => {
            let msg = MessageOpen::from_bytes(packet.payload(), len)?;
            println!("MessageOpen {:?}", msg);
            return Ok(Message::Open(msg));
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
    Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe).into())
}

impl Decoder for Bgp {
    type Item = Message;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> std::io::Result<Option<Message>> {
        match from_bytes(src) {
            Ok(m) => {
                println!("Packet length {}", m.len());
                let _ = src.split_to(m.len());
                Ok(Some(m))
            }
            Err(_) => Ok(None),
        }
    }
}
