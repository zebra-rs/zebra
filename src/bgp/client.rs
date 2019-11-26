#![allow(dead_code)]
use std::io::Read;
use std::net::SocketAddrV4;
use std::net::TcpStream;

pub struct Client<'a> {
    addr: &'a str,
    stream: Option<TcpStream>,
}

impl<'a> Client<'a> {
    pub fn new(addr: &'a str) -> Self {
        Client {
            addr: addr,
            stream: None,
        }
    }

    pub fn connect(&self) -> Result<(), failure::Error> {
        let sock_addr_str = format!("{}:{}", self.addr, super::BGP_PORT);
        let sock_addr: SocketAddrV4 = sock_addr_str.parse()?;

        let mut stream = TcpStream::connect(sock_addr)?;

        // Read BGP message.
        let mut buf = [0u8; 4096];
        let n = stream.read(&mut buf)?;

        if n == 0 {
            println!("BGP socket closed");
            std::process::exit(1);
        }

        // Mimimum BGP message len is 19.
        if n < 19 {
            // Need to read more.
            println!("BGP packet length is smaller than minimum length (19).");
            std::process::exit(1);
        }
        println!("Read num: {}", n);

        let packet = crate::bgp::packet::BgpPacket::new(&buf).unwrap();
        let typ = packet.get_bgp_type();
        let length = packet.get_length();

        use crate::bgp::packet::BgpTypes;
        use pnet::packet::Packet;

        println!("Type {:?}", typ);
        match typ {
            BgpTypes::OPEN => {
                println!("Open message!");
                let open = crate::bgp::packet::BgpOpenPacket::new(packet.payload()).unwrap();
                println!("Version: {:?}", open.get_version());
                println!("AS: {:?}", open.get_asn());
                println!("HoldTime: {:?}", open.get_hold_time());
                println!("OptParamLen: {:?}", open.get_opt_param_len());
                println!("Payload Len: {:?}", open.payload().len());
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
        Ok(())
    }
}
