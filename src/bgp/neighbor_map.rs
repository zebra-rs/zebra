#![allow(dead_code)]

use super::Neighbor;
use std::collections::BTreeMap;
use std::net::IpAddr;

pub struct NeighborMap(BTreeMap<IpAddr, Neighbor>);

impl NeighborMap {
    pub fn new() -> Self {
        NeighborMap(BTreeMap::<IpAddr, Neighbor>::new())
    }
}

use std::ops::{Deref, DerefMut};

impl Deref for NeighborMap {
    type Target = BTreeMap<IpAddr, Neighbor>;
    fn deref(&self) -> &BTreeMap<IpAddr, Neighbor> {
        &self.0
    }
}

impl DerefMut for NeighborMap {
    fn deref_mut(&mut self) -> &mut BTreeMap<IpAddr, Neighbor> {
        &mut self.0
    }
}

#[cfg(test)]
mod test {
    use super::Neighbor;
    use super::NeighborMap;
    use std::net::IpAddr;

    #[test]
    fn insert() {
        let mut v = NeighborMap::new();

        let n1 = Neighbor {
            ipaddr: "192.168.55.1".parse::<IpAddr>().unwrap(),
        };
        let n2 = Neighbor {
            ipaddr: "192.168.55.2".parse().unwrap(),
        };
        let n3 = Neighbor {
            ipaddr: "10.0.0.1".parse().unwrap(),
        };
        let n4 = Neighbor {
            ipaddr: "192.168.55.2".parse().unwrap(),
        };
        let n5 = Neighbor {
            ipaddr: "::1".parse::<IpAddr>().unwrap(),
        };

        if let Some(_) = v.insert(n1.ipaddr, n1) {
            panic!("n1 is already inserted");
        }
        if let Some(_) = v.insert(n2.ipaddr, n2) {
            panic!("n2 is already inserted");
        }
        if let Some(_) = v.insert(n3.ipaddr, n3) {
            panic!("n3 is already inserted");
        }
        assert_eq!(v.len(), 3);

        if let None = v.insert(n4.ipaddr, n4) {
            panic!("n4 is not inserted");
        }
        assert_eq!(v.len(), 3);

        let addr1: std::net::IpAddr = "10.0.0.1".parse().unwrap();
        if let Some(n) = v.get(&addr1) {
            println!("Found neighbor: {}", n.ipaddr);
            assert_eq!(addr1, n.ipaddr);
        } else {
            panic!("can't find 10.0.0.1");
        }

        if let Some(_) = v.insert(n5.ipaddr, n5) {
            panic!("n5 is already inserted");
        }
        assert_eq!(v.len(), 4);
    }

    #[test]
    fn insert2() {
        let mut v = NeighborMap::new();
        let addr1: IpAddr = "192.168.55.1".parse::<IpAddr>().unwrap();
        let addr2: IpAddr = "192.168.55.1".parse::<IpAddr>().unwrap();
        let n1 = Neighbor { ipaddr: addr1 };
        let n2 = Neighbor { ipaddr: addr2 };

        let ret = v.insert(addr1, n1);
        match ret {
            Some(_) => {
                panic!("it has old value");
            }
            None => {
                println!("new value");
            }
        }

        let ret = v.insert(addr2, n2);
        match ret {
            Some(_) => {
                println!("it has old value");
            }
            None => {
                panic!("new value");
            }
        }
    }
}
