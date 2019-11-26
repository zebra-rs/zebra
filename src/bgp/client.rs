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

        let packet = crate::bgp::packet::bgp::BgpPacket::new(&buf).unwrap();
        let typ = packet.get_bgp_type();
        let length = packet.get_length();

        use crate::bgp::packet::BgpTypes;
        println!("Type {:?}", typ);
        match typ {
            BgpTypes::OPEN => {
                println!("Open message!");
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
        //println!("Payload {:?}", payload);

        return Ok(());
    }
}
