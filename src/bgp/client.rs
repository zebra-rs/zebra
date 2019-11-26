#![allow(dead_code)]
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

    // fn cap_parse(&self, packet: &'a [u8]) -> (&'a [u8]) {
    //     println!("cap_parse len: {}", packet.len());

    //     let cap = crate::bgp::packet::BgpOpenOptPacket::new(packet).unwrap();
    //     println!("cap type {}", cap.get_typ());
    //     println!("cap length {}", cap.get_length());
    //     println!("cap payload Len: {:?}", cap.payload().len());
    //     let ret = cap.payload();
    //     (&ret)
    // }

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
        let buf = &buf[0..n];

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

                    // 2 is capability.
                    if opt.get_typ() != 2 {
                        return Ok(());
                    }

                    if opt.get_length() > 0 {
                        let cap = crate::bgp::packet::BgpOpenOptPacket::new(opt.payload()).unwrap();
                        println!("cap type {}", cap.get_typ());
                        println!("cap length {}", cap.get_length());
                        println!("cap payload Len: {:?}", cap.payload().len());

                        let mut payload = cap.payload();
                        let offset: usize = cap.get_length() as usize;
                        payload = &payload[offset..];
                        println!("cap payload Len(adj): {:?}", payload.len());
                        //let mut payload_len = payload.len();

                        //self.cap_parse(payload);

                        // while payload_len > 0 {
                        //     let cap = crate::bgp::packet::BgpOpenOptPacket::new(payload).unwrap();
                        //     println!("cap type {}", cap.get_typ());
                        //     println!("cap length {}", cap.get_length());
                        //     println!("cap payload Len: {:?}", cap.payload().len());
                        //     payload = cap.payload();
                        //     let offset: usize = cap.get_length() as usize;
                        //     payload = &payload[offset..];
                        //     println!("cap payload Len(adj): {:?}", payload.len());
                        //     payload_len = payload.len();
                        // }
                    }
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
