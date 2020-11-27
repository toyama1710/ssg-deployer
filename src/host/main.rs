mod dephs;
use deployer::tcp_wrap::*;
use deployer::util;
use std::net::*;

fn main() {
    let (p, sections) = dephs::get_config().unwrap();
    let addr = format!("0.0.0.0:{}", p);
    let listner = TcpListener::bind(addr).unwrap();

    for stream in listner.incoming() {
        let mut stream = stream.unwrap();

        println!("connection established");

        let mut msg = Vec::new();
        stream.read_msg(&mut msg).unwrap();
        let msg = String::from_utf8(msg).unwrap();

        let sec = sections.iter().find(|v| v.name == msg);
        if sec.is_none() {
            eprintln!("section not found");
            panic!();
        }
        let sec = sec.unwrap();

        if let Err(e) = util::auth(&mut stream, &sec.own_pri, &sec.client_pub) {
            eprintln!("authentication failed");
            panic!("{:?}", e);
        }

        let aes_key;
        match util::exchange_aes_key(&mut stream, &sec.own_pri, &sec.client_pub) {
            Err(e) => {
                eprintln!("authentication failed");
                panic!("{:?}", e);
            }
            Ok(v) => {
                aes_key = v;
            }
        }
    }
}
