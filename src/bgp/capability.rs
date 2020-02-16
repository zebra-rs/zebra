#![allow(dead_code)]

const AFI_IP: u16 = 1;
const AFI_IP6: u16 = 2;
const AFI_L2VPN: u16 = 25;
const AFI_OPAQUE: u16 = 16397;

const SAFI_UNICAST: u8 = 1;
const SAFI_MULTICAST: u8 = 2;
const SAFI_MPLS_LABEL: u8 = 4;
const SAFI_ENCAPSULATION: u8 = 7;
const SAFI_VPLS: u8 = 65;
const SAFI_EVPN: u8 = 70;
const SAFI_MPLS_VPN: u8 = 128;
const SAFI_MPLS_VPN_MULTICAST: u8 = 129;
const SAFI_ROUTE_TARGET_CONSTRAINTS: u8 = 132;
const SAFI_FLOW_SPEC_UNICAST: u8 = 133;
const SAFI_FLOW_SPEC_VPN: u8 = 134;
const SAFI_KEY_VALUE: u8 = 241;

const CAPABILITY_CODE_MP: u8 = 1; /* Multiprotocol Extensions */
const CAPABILITY_CODE_REFRESH: u8 = 2; /* Route Refresh Capability */
const CAPABILITY_CODE_ORF: u8 = 3; /* Cooperative Route Filtering Capability */
const CAPABILITY_CODE_LABEL_INFO: u8 = 4; /* Carrying Label Information */
const CAPABILITY_CODE_ENHE: u8 = 5; /* Extended Next Hop Encoding */
const CAPABILITY_CODE_RESTART: u8 = 64; /* Graceful Restart Capability */
const CAPABILITY_CODE_AS4: u8 = 65; /* 4-octet AS number Capability */
const CAPABILITY_CODE_DYNAMIC_OLD: u8 = 66; /* Dynamic Capability, deprecated since 2003 */
const CAPABILITY_CODE_DYNAMIC: u8 = 67; /* Dynamic Capability */
const CAPABILITY_CODE_ADDPATH: u8 = 69; /* Addpath Capability */
const CAPABILITY_CODE_ENH_REFRESH: u8 = 70; /* Enhanced Route Refresh */
const CAPABILITY_CODE_FQDN: u8 = 73; /* Advertise hostname capability */
const CAPABILITY_CODE_REFRESH_CISCO: u8 = 128; /* Route Refresh Capability(Cisco) */
const CAPABILITY_CODE_LLGR: u8 = 129; /* Long Lived Graceful Restart */
const CAPABILITY_CODE_ORF_OLD: u8 = 130; /* Cooperative Route Filtering Capability(Cisco) */

#[derive(Debug)]
pub struct Capabilities(Vec<Capability>);

impl Capabilities {
    pub fn new() -> Self {
        Capabilities(Vec::<Capability>::new())
    }

    pub fn push(&mut self, value: Capability) {
        self.0.push(value)
    }
}

#[derive(Debug)]
pub struct RestartFlags {
    restart_state: u8,
    restart_time: u16,
    afi: u16,
    safi: u8,
    forwarding_state: u8,
}

#[derive(Debug)]
pub enum Capability {
    Refresh,
    Mp(u16, u8),
    As4(u32),
    Restart(Vec<RestartFlags>),
}

pub fn capability_parse<'a>(packet: &'a [u8], caps: &mut Capabilities) -> Option<&'a [u8]> {
    if packet.len() == 0 {
        return None;
    }
    if packet.len() < 2 {
        return None;
    }
    let typ = packet[0];
    let length = packet[1];
    println!("type: {} length: {}", typ, length);

    match typ {
        CAPABILITY_CODE_REFRESH => {
            if length != 0 {
                return None;
            }
            // println!("Capability: Refresh");
            caps.push(Capability::Refresh);
        }
        CAPABILITY_CODE_MP => {
            if length != 4 {
                return None;
            }
            // AFI and SAFI.
            let afi: u16 = u16::from_be_bytes([packet[2], packet[3]]);
            let _res: u8 = packet[4];
            let safi: u8 = packet[5];

            // println!("Capability: MP AFI={} SAFI={}", afi, safi,);
            caps.push(Capability::Mp(afi, safi));
        }
        CAPABILITY_CODE_AS4 => {
            if length != 4 {
                return None;
            }
            let asn: u32 = u32::from_be_bytes([packet[2], packet[3], packet[4], packet[5]]);
            // println!("Capability: AS4 {}", asn);
            caps.push(Capability::As4(asn));
        }
        CAPABILITY_CODE_RESTART => {
            if (length % 6) != 0 {
                return None;
            }
            println!("Capability: Restart");
        }
        CAPABILITY_CODE_LLGR => {
            println!("Capability: LLGR");
        }
        CAPABILITY_CODE_ADDPATH => {
            println!("Capability: AddPath");
        }
        _ => println!("other capability"),
    }

    let offset: usize = 2 + (length as usize);
    Some(&packet[offset..])
}
