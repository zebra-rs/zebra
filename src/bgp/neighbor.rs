#![allow(dead_code)]
pub struct Neighbor {
    pub ipaddr: std::net::IpAddr,
}

impl std::cmp::Ord for Neighbor {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ipaddr.cmp(&other.ipaddr)
    }
}

impl std::cmp::PartialOrd for Neighbor {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.ipaddr.partial_cmp(&other.ipaddr)
    }
}

impl std::cmp::PartialEq for Neighbor {
    fn eq(&self, other: &Self) -> bool {
        self.ipaddr == other.ipaddr
    }
}

impl std::cmp::Eq for Neighbor {}

pub struct NeighborVec(Vec<Neighbor>);

impl NeighborVec {
    pub fn new() -> Self {
        NeighborVec(Vec::<Neighbor>::new())
    }
}

use std::ops::{Deref, DerefMut};

impl Deref for NeighborVec {
    type Target = Vec<Neighbor>;
    fn deref(&self) -> &Vec<Neighbor> {
        &self.0
    }
}

impl DerefMut for NeighborVec {
    fn deref_mut(&mut self) -> &mut Vec<Neighbor> {
        &mut self.0
    }
}

impl NeighborVec {
    pub fn insert_sort(&mut self, n: Neighbor) -> Option<()> {
        if let Err(pos) = self.binary_search(&n) {
            self.insert(pos, n);
            Some(())
        } else {
            None
        }
        // match self.binary_search(&n) {
        //     Ok(_) => None,
        //     Err(pos) => {
        //         self.insert(pos, n);
        //         Some(())
        //     }
        // }
        // let pos = v.binary_search(&n).unwrap_or_else(|x| x);
        // v.insert(pos, n3);
    }
}

#[cfg(test)]
mod test {
    use super::Neighbor;
    use super::NeighborVec;
    use std::net::IpAddr;

    #[test]
    fn sort_add() {
        let mut v = NeighborVec::new();

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

        assert_eq!(v.insert_sort(n1).unwrap(), ());
        assert_eq!(v.insert_sort(n2).unwrap(), ());
        assert_eq!(v.insert_sort(n3).unwrap(), ());
        assert_eq!(v.insert_sort(n4), None);

        assert_eq!(v.len(), 3);

        let nr1 = &v[0];
        let addr1: std::net::IpAddr = "10.0.0.1".parse().unwrap();
        assert_eq!(nr1.ipaddr, addr1);

        assert_eq!(v.insert_sort(n5).unwrap(), ());
        assert_eq!(v.len(), 4);
    }
}
