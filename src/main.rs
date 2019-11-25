fn main() {
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
