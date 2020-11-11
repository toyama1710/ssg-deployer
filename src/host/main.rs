mod dephs;
use std::net::*;

fn main() {
    let p = dephs::get_config();
    let addr = format!("0.0.0.0:{}", p);
    let listner = TcpListener::bind(addr).unwrap();

    for stream in listner.incoming() {
        let stream = stream.unwrap();

        println!("connection established");
    }
}
