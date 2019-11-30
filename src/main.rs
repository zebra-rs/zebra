fn main() {
    use std::net::IpAddr;
    use zebra::bgp::Neighbor;
    use zebra::bgp::NeighborMap;

    let mut v = NeighborMap::new();
    let addr1: IpAddr = "192.168.55.1".parse::<IpAddr>().unwrap();
    let addr2: IpAddr = "192.168.55.1".parse::<IpAddr>().unwrap();
    let n1 = Neighbor { ipaddr: addr1 };
    let n2 = Neighbor { ipaddr: addr2 };

    let ret = v.insert(addr1, n1);
    match ret {
        Some(_) => {
            println!("it has old value");
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
            println!("new value");
        }
    }

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Please specify BGP neighbor IP address");
        std::process::exit(1);
    }

    let client = zebra::bgp::Client::new(&args[1]);
    if let Err(err) = client.connect() {
        println!("{}", err);
        std::process::exit(1);
    }
    println!("client connect success");
}
