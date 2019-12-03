#![allow(dead_code)]

const COMMUNITY_INTERNET: u32 = 0x0;
const COMMUNITY_GSHUT: u32 = 0xFFFF0000;
const COMMUNITY_ACCEPT_OWN: u32 = 0xFFFF0001;
const COMMUNITY_FILTER_TRANSLATED_V4: u32 = 0xFFFF0002;
const COMMUNITY_FILTER_V4: u32 = 0xFFFF0003;
const COMMUNITY_FILTER_TRANSLATED_V6: u32 = 0xFFFF0004;
const COMMUNITY_FILTER_V6: u32 = 0xFFFF0005;
const COMMUNITY_LLGR_STALE: u32 = 0xFFFF0006;
const COMMUNITY_NO_LLGR: u32 = 0xFFFF0007;
const COMMUNITY_ACCEPT_OWN_NEXTHOP: u32 = 0xFFFF0008;
const COMMUNITY_BLACKHOLE: u32 = 0xFFFF029A;
const COMMUNITY_NO_EXPORT: u32 = 0xFFFFFF01;
const COMMUNITY_NO_ADVERTISE: u32 = 0xFFFFFF02;
const COMMUNITY_NO_EXPORT_SUBCONFED: u32 = 0xFFFFFF03;
const COMMUNITY_LOCAL_AS: u32 = 0xFFFFFF03;
const COMMUNITY_NO_PEER: u32 = 0xFFFFFF04;

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref COMMUNITY_STR_MAP: Mutex<HashMap<&'static str, u32>> = {
        let mut m = HashMap::new();
        m.insert("internet", COMMUNITY_INTERNET);
        m.insert("graceful-shutdown", COMMUNITY_GSHUT);
        m.insert("accept-own", COMMUNITY_ACCEPT_OWN);
        m.insert("route-filter-translated-v4", COMMUNITY_FILTER_TRANSLATED_V4);
        m.insert("route-filter-v4", COMMUNITY_FILTER_V4);
        m.insert("route-filter-translated-v6", COMMUNITY_FILTER_TRANSLATED_V6);
        m.insert("route-filter-v6", COMMUNITY_FILTER_V6);
        m.insert("llgr-stale", COMMUNITY_LLGR_STALE);
        m.insert("no-llgr", COMMUNITY_NO_LLGR);
        m.insert("accept-own-nexthop", COMMUNITY_ACCEPT_OWN_NEXTHOP);
        m.insert("blackhole", COMMUNITY_BLACKHOLE);
        m.insert("no-export", COMMUNITY_NO_EXPORT);
        m.insert("no-advertise", COMMUNITY_NO_ADVERTISE);
        m.insert("local-AS", COMMUNITY_LOCAL_AS);
        m.insert("no-peer", COMMUNITY_NO_PEER);
        Mutex::new(m)
    };
}

lazy_static! {
    static ref COMMUNITY_VAL_MAP: Mutex<HashMap<u32, &'static str>> = {
        let mut m = HashMap::new();
        m.insert(COMMUNITY_INTERNET, "internet");
        m.insert(COMMUNITY_GSHUT, "graceful-shutdown");
        m.insert(COMMUNITY_ACCEPT_OWN, "accept-own");
        m.insert(COMMUNITY_FILTER_TRANSLATED_V4, "route-filter-translated-v4");
        m.insert(COMMUNITY_FILTER_V4, "route-filter-v4");
        m.insert(COMMUNITY_FILTER_TRANSLATED_V6, "route-filter-translated-v6");
        m.insert(COMMUNITY_FILTER_V6, "route-filter-v6");
        m.insert(COMMUNITY_LLGR_STALE, "llgr-stale");
        m.insert(COMMUNITY_NO_LLGR, "no-llgr");
        m.insert(COMMUNITY_ACCEPT_OWN_NEXTHOP, "accept-own-nexthop");
        m.insert(COMMUNITY_BLACKHOLE, "blackhole");
        m.insert(COMMUNITY_NO_EXPORT, "no-export");
        m.insert(COMMUNITY_NO_ADVERTISE, "no-advertise");
        m.insert(COMMUNITY_LOCAL_AS, "local-AS");
        m.insert(COMMUNITY_NO_PEER, "no-peer");
        Mutex::new(m)
    };
}

pub struct Communities(Vec<u32>);

impl Communities {
    pub fn new() -> Self {
        Communities(Vec::<u32>::new())
    }
}

use std::ops::{Deref, DerefMut};

impl Deref for Communities {
    type Target = Vec<u32>;
    fn deref(&self) -> &Vec<u32> {
        &self.0
    }
}

impl DerefMut for Communities {
    fn deref_mut(&mut self) -> &mut Vec<u32> {
        &mut self.0
    }
}

use std::fmt;
impl fmt::Display for Communities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let map = COMMUNITY_VAL_MAP.lock().unwrap();

        let format = |v: &u32| {
            let hval: u32 = (v & 0xFFFF0000) >> 16;
            let lval: u32 = v & 0x0000FFFF;
            hval.to_string() + ":" + &lval.to_string()
        };

        let mut iter = self.0.iter();
        let val = match iter.next() {
            None => String::new(),
            Some(first_elem) => {
                let mut result = match map.get(first_elem) {
                    Some(&s) => s.to_string(),
                    None => format(first_elem),
                };
                for elem in iter {
                    result.push_str(" ");
                    let elem_str = match map.get(elem) {
                        Some(&s) => s.to_string(),
                        None => format(elem),
                    };
                    result = result + &elem_str;
                }
                result
            }
        };
        write!(f, "{}", val)
    }
}

impl Communities {
    pub fn parse_community(s: &str) -> Option<u32> {
        let com_strs: Vec<&str> = s.split(':').collect();
        match com_strs.len() {
            // ASN:NN format.
            2 => {
                if let Ok(hval) = com_strs[0].parse::<u16>() {
                    if let Ok(lval) = com_strs[1].parse::<u16>() {
                        return Some(u32::from(hval) << 16 | u32::from(lval));
                    }
                }
                None
            }
            // NN format.
            1 => {
                if let Ok(val) = com_strs[0].parse::<u32>() {
                    return Some(val);
                }
                None
            }
            // Otherwise none.
            _ => None,
        }
    }

    pub fn from_str(s: &str) -> Option<Communities> {
        let com_strs: Vec<&str> = s.split(' ').collect();
        if com_strs.len() == 0 {
            return None;
        }

        // At least one community string exists.
        let mut coms = Communities::new();
        let map = COMMUNITY_STR_MAP.lock().unwrap();

        for s in com_strs.iter() {
            // Well known communit value match.
            match map.get(s) {
                Some(&c) => coms.push(c),
                None => {
                    // ASN:NN or NN format parse.
                    if let Some(c) = Communities::parse_community(s) {
                        coms.push(c)
                    } else {
                        return None;
                    }
                }
            }
        }
        return Some(coms);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn push() {
        let mut com = Communities::new();
        com.push(1u32);
        com.push(2u32);
        com.push(3u32);
        assert_eq!(format!("{}", com), "0:1 0:2 0:3");

        let mut com = Communities::new();
        com.push(1u32);
        com.push(COMMUNITY_BLACKHOLE);
        com.push(3u32);
        assert_eq!(format!("{}", com), "0:1 blackhole 0:3");
    }

    #[test]
    fn from_str() {
        let com = Communities::from_str("no-export 100:10 100").unwrap();
        assert_eq!(format!("{}", com), "no-export 100:10 0:100");

        let com = Communities::from_str("100:10 no-export 100").unwrap();
        assert_eq!(format!("{}", com), "100:10 no-export 0:100");

        let com = Communities::from_str("100 no-export 100:10").unwrap();
        assert_eq!(format!("{}", com), "0:100 no-export 100:10");

        let com = Communities::from_str("4294967295 no-export 100:10").unwrap();
        assert_eq!(format!("{}", com), "65535:65535 no-export 100:10");

        let com = Communities::from_str("4294967296 no-export 100:10");
        if let Some(_) = com {
            panic!("com must be None");
        }

        let com = Communities::from_str("4294967296 not-well-defined 100:10");
        if let Some(_) = com {
            panic!("com must be None");
        }

        let com = Communities::from_str("");
        if let Some(_) = com {
            panic!("com must be None");
        }

        let com = Communities::from_str("-1");
        if let Some(_) = com {
            panic!("com must be None");
        }

        let com = Communities::from_str("100:test");
        if let Some(_) = com {
            panic!("com must be None");
        }

        let com = Communities::from_str("65535:65535").unwrap();
        assert_eq!(format!("{}", com), "65535:65535");

        let com = Communities::from_str("65535:65536");
        if let Some(_) = com {
            panic!("com must be None");
        }

        let com = Communities::from_str("65536:65535");
        if let Some(_) = com {
            panic!("com must be None");
        }

        let com = Communities::from_str("0").unwrap();
        assert_eq!(format!("{}", com), "internet");

        let com = Communities::from_str("1").unwrap();
        assert_eq!(format!("{}", com), "0:1");

        let com = Communities::from_str("no-export 100:10 100").unwrap();
        if !com.contains(&COMMUNITY_NO_EXPORT) {
            panic!("Community must contain no-export");
        }

        if com.contains(&COMMUNITY_NO_ADVERTISE) {
            panic!("Community must not contain no-advertise");
        }

        let val = Communities::parse_community("100:10").unwrap();
        if !com.contains(&val) {
            panic!("Community must contain 100:10");
        }
    }

    #[test]
    fn to_string() {}
}
