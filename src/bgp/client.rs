#![allow(dead_code)]
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

        let _conn = TcpStream::connect(sock_addr)?;

        return Ok(());
    }
}
