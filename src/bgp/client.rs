#![allow(dead_code)]
use super::*;
use pnet::packet::Packet;
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

    pub fn send_open(&self) {
        let mut buf = [0u8; 4096];
        let mut packet = crate::bgp::packet::MutableBgpPacket::new(&mut buf).unwrap();

        use crate::bgp::packet::BgpTypes;
        let marker: crate::bgp::packet::bgp::Marker = crate::bgp::packet::bgp::Marker::new(
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff,
        );
        packet.set_marker(marker);
        packet.set_bgp_type(BgpTypes::OPEN);
        packet.set_length(19u16);

        let p = packet.packet();
        println!("send_open length {}", p.len());
        for i in 0..19 {
            println!("{}: {}", i, p[i]);
        }
    }

    pub fn connect(&self) -> Result<(), failure::Error> {
        let sock_addr_str = format!("{}:{}", self.addr, BGP_PORT);
        let sock_addr: SocketAddrV4 = sock_addr_str.parse()?;

        let mut stream = TcpStream::connect(sock_addr)?;

        // Send BGP packet.
        self.send_open();

        // Read BGP message.
        let mut buf = [0u8; 4096];
        let n = stream.read(&mut buf)?;

        if n == 0 {
            println!("BGP socket closed");
            std::process::exit(1);
        }
        let buf = &buf[0..n];

        // Minimum BGP message len is 19.
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

        println!("Type {:?}", typ);
        match typ {
            BgpTypes::OPEN => {
                println!("Open message!");
                let open = crate::bgp::packet::BgpOpenPacket::new(packet.payload()).unwrap();
                println!("Version: {:?}", open.get_version());
                println!("AS: {:?}", open.get_asn());
                println!("HoldTime: {:?}", open.get_hold_time());
                let opt_param_len = open.get_opt_param_len();
                println!("OptParamLen: {:?}", opt_param_len);
                println!("Payload Len: {:?}", open.payload().len());

                // Open message.
                if opt_param_len > 0 {
                    println!("parse opt param");
                    let opt = crate::bgp::packet::BgpOpenOptPacket::new(open.payload()).unwrap();
                    println!("opt type {}", opt.get_typ());
                    println!("opt length {}", opt.get_length());
                    println!("opt payload len {}", opt.payload().len());

                    // When Open opt message is not capability(2) return here.
                    if opt.get_typ() != 2 {
                        return Ok(());
                    }

                    // Parse Open capability message.
                    let mut packet: &[u8] = opt.payload();
                    let mut caps = crate::bgp::Capabilities::new();
                    while let Some(payload) = capability_parse(packet, &mut caps) {
                        packet = payload;
                        println!("len {}", packet.len());
                    }
                    caps.dump();
                }
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
