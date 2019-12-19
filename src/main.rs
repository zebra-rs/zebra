use tokio::runtime::Runtime;
use zebra::bgp::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let com = Communities::from_str("no-export 100:10 100").unwrap();
    println!("{}", com.to_json());

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Please specify BGP neighbor IP address");
        std::process::exit(1);
    }

    let _rt = Runtime::new()?;

    let client = zebra::bgp::Client::new(&args[1]);
    if let Err(err) = client.connect() {
        println!("{}", err);
        std::process::exit(1);
    }
    println!("client connect success");

    Ok(())
}
