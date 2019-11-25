fn main() {
    let com = zebra::bgp::Communities::from_str("no-export 100:10 100").unwrap();
    println!("Communities: {}", com);
}
