#![allow(dead_code)]

use byteorder::WriteBytesExt;
use byteorder::{NetworkEndian, ReadBytesExt};
use std::io::Cursor;

pub const AFI_IP: u16 = 1;
const AFI_IP6: u16 = 2;
const AFI_L2VPN: u16 = 25;
const AFI_OPAQUE: u16 = 16397;

const SAFI_UNICAST: u8 = 1;
const SAFI_MULTICAST: u8 = 2;
const SAFI_MPLS_LABEL: u8 = 4;
const SAFI_ENCAPSULATION: u8 = 7;
const SAFI_VPLS: u8 = 65;
const SAFI_EVPN: u8 = 70;
pub const SAFI_MPLS_VPN: u8 = 128;
const SAFI_MPLS_VPN_MULTICAST: u8 = 129;
const SAFI_ROUTE_TARGET_CONSTRAINTS: u8 = 132;
const SAFI_FLOW_SPEC_UNICAST: u8 = 133;
const SAFI_FLOW_SPEC_VPN: u8 = 134;
const SAFI_KEY_VALUE: u8 = 241;

#[derive(Debug)]
pub struct Capabilities(Vec<Capability>);

impl Capabilities {
    pub fn new() -> Self {
        Capabilities(Vec::<Capability>::new())
    }

    pub fn push(&mut self, value: Capability) {
        self.0.push(value)
    }

    pub fn get_ref(&self) -> &Vec<Capability> {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(failure::Fail, Debug)]
pub enum Error {
    #[fail(display = "malformed packet")]
    Malformed,
}

#[derive(Debug)]
pub struct Family {
    pub afi: u16,
    pub safi: u8,
}

#[derive(Debug)]
pub struct RestartFlags {
    restart_state: u8,
    restart_time: u16,
    family: Family,
    forwarding_state: u8,
}

#[derive(Debug)]
pub enum Capability {
    RouteRefresh,
    MultiProtocol(Family),
    FourOctetAs(u32),
    GracefulRestart {
        flags: u8,
        time: u16,
        families: Vec<(Family, u8)>,
    },
    DynamicCapability,
    LongLived(Vec<(Family, u8, u32)>),
    AddPath(Vec<(Family, u8)>),
}

impl Capability {
    const MULTI_PROTOCOL: u8 = 1; /* Multiprotocol Extensions */
    const ROUTE_REFRESH: u8 = 2; /* Route Refresh Capability */
    const GRACEFUL_RESTART: u8 = 64; /* Graceful Restart Capability */
    const FOUR_OCTET_AS: u8 = 65; /* 4-octet AS number Capability */
    const DYNAMIC_CAPABILITY_OLD: u8 = 66; /* Dynamic Capability, deprecated since 2003 */
    const DYNAMIC_CAPABILITY: u8 = 67; /* Dynamic Capability */
    const ADD_PATH: u8 = 69; /* Addpath Capability */
    const LONG_LIVED_GRACEFUL_RESTART: u8 = 129; /* Long Lived Graceful Restart */
    const ROUTE_REFRESH_CISCO: u8 = 128; /* Route Refresh Capability(Cisco) */

    const CAPABILITY_CODE_ORF: u8 = 3; /* Cooperative Route Filtering Capability */
    const CAPABILITY_CODE_LABEL_INFO: u8 = 4; /* Carrying Label Information */
    const CAPABILITY_CODE_ENHE: u8 = 5; /* Extended Next Hop Encoding */
    const CAPABILITY_CODE_ENH_REFRESH: u8 = 70; /* Enhanced Route Refresh */
    const CAPABILITY_CODE_FQDN: u8 = 73; /* Advertise hostname capability */
    const CAPABILITY_CODE_ORF_OLD: u8 = 130; /* Cooperative Route Filtering Capability(Cisco) */

    pub fn from_bytes(c: &mut Cursor<&[u8]>) -> Result<Capability, failure::Error> {
        let code = c.read_u8()?;
        let len = c.read_u8()?;
        println!("code {} len {}", code, len);

        match code {
            Capability::MULTI_PROTOCOL => {
                if len != 4 {
                    return Err(Error::Malformed.into());
                }
                let afi: u16 = c.read_u16::<NetworkEndian>()?;
                let _res: u8 = c.read_u8()?;
                let safi: u8 = c.read_u8()?;
                return Ok(Capability::MultiProtocol(Family { afi, safi }));
            }
            Capability::ROUTE_REFRESH | Capability::ROUTE_REFRESH_CISCO => {
                if len != 0 {
                    return Err(Error::Malformed.into());
                }
                return Ok(Capability::RouteRefresh);
            }
            Capability::GRACEFUL_RESTART => {
                if len < 6 || (len - 2) % 4 != 0 {
                    return Err(Error::Malformed.into());
                }
                let restart = c.read_u16::<NetworkEndian>()?;
                let flags = (restart >> 12) as u8;
                let time = restart & 0xfff;

                let mut families = Vec::new();
                let mut num_families = (len - 2) / 4;
                while num_families > 0 {
                    let afi: u16 = c.read_u16::<NetworkEndian>()?;
                    let safi: u8 = c.read_u8()?;
                    let flags: u8 = c.read_u8()?;
                    families.push((Family { afi, safi }, flags));
                    num_families -= 1;
                }
                return Ok(Capability::GracefulRestart {
                    flags,
                    time,
                    families,
                });
            }
            Capability::FOUR_OCTET_AS => {
                if len != 4 {
                    return Err(Error::Malformed.into());
                }
                let asn: u32 = c.read_u32::<NetworkEndian>()?;
                return Ok(Capability::FourOctetAs(asn));
            }
            Capability::DYNAMIC_CAPABILITY | Capability::DYNAMIC_CAPABILITY_OLD => {
                if len != 0 {
                    return Err(Error::Malformed.into());
                }
                return Ok(Capability::DynamicCapability);
            }
            Capability::ADD_PATH => {
                if len < 4 || len % 4 != 0 {
                    return Err(Error::Malformed.into());
                }
                let mut v = Vec::new();
                let mut num = len / 4;
                while num > 0 {
                    let afi = c.read_u16::<NetworkEndian>()?;
                    let safi = c.read_u8()?;
                    let flags = c.read_u8()?;
                    v.push((Family { afi, safi }, flags));
                    num -= 1;
                }
                return Ok(Capability::AddPath(v));
            }
            Capability::LONG_LIVED_GRACEFUL_RESTART => {
                if len < 7 || len % 7 != 0 {
                    return Err(Error::Malformed.into());
                }
                let mut v = Vec::new();
                let mut num = len / 7;
                while num > 0 {
                    let afi = c.read_u16::<NetworkEndian>()?;
                    let safi = c.read_u8()?;
                    let flags = c.read_u8()?;
                    let time = (c.read_u8()? as u32) << 16
                        | (c.read_u8()? as u32) << 8
                        | c.read_u8()? as u32;
                    v.push((Family { afi, safi }, flags, time));
                    num -= 1;
                }
                return Ok(Capability::LongLived(v));
            }
            _ => {}
        }
        Ok(Capability::RouteRefresh)
    }

    pub fn to_bytes(&self, c: &mut Cursor<&mut [u8]>) -> Result<usize, failure::Error> {
        let sp = c.position();
        match self {
            Capability::MultiProtocol(family) => {
                c.write_u8(Capability::MULTI_PROTOCOL)?;
                c.write_u8(4)?;
                c.write_u16::<NetworkEndian>(family.afi)?;
                c.write_u8(0)?;
                c.write_u8(family.safi)?;
                return Ok((c.position() - sp) as usize);
            }
            _ => {}
        }
        Ok(0)
    }
}

// This is left for sample of other methods.
// capability_parse() caller side:
//
// while let Some(payload) = capability_parse(buf, &mut caps) {
//     buf = payload;
//     println!("len {}", buf.len());
// }
pub fn capability_parse<'a>(packet: &'a [u8], caps: &mut Capabilities) -> Option<&'a [u8]> {
    if packet.len() < 2 {
        return None;
    }

    let code = packet[0];
    let length = packet[1];
    let offset: usize = 2 + (length as usize);

    if packet.len() < offset {
        return None;
    }

    match code {
        Capability::ROUTE_REFRESH => {
            if length != 0 {
                return None;
            }
            caps.push(Capability::RouteRefresh);
        }
        Capability::MULTI_PROTOCOL => {
            if length != 4 {
                return None;
            }
            // AFI and SAFI.
            let afi: u16 = u16::from_be_bytes([packet[2], packet[3]]);
            let _res: u8 = packet[4];
            let safi: u8 = packet[5];

            caps.push(Capability::MultiProtocol(Family { afi, safi }));
        }
        Capability::FOUR_OCTET_AS => {
            if length != 4 {
                return None;
            }
            let asn: u32 = u32::from_be_bytes([packet[2], packet[3], packet[4], packet[5]]);
            caps.push(Capability::FourOctetAs(asn));
        }
        Capability::GRACEFUL_RESTART => {
            if (length % 6) != 0 {
                return None;
            }
            println!("Capability: Restart");
        }
        Capability::LONG_LIVED_GRACEFUL_RESTART => {
            println!("Capability: LLGR");
        }
        Capability::ADD_PATH => {
            println!("Capability: AddPath");
        }
        _ => println!("Capability: Other values"),
    }

    Some(&packet[offset..])
}
