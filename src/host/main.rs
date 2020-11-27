mod dephs;
use deployer::tcp_wrap::*;
use deployer::util;
use std::io::Write;
use std::net::*;
use std::path::Path;

fn main() {
    let p = dephs::get_config();
    let addr = format!("0.0.0.0:{}", p);
    let listner = TcpListener::bind(addr).unwrap();

    for stream in listner.incoming() {
        let mut stream = stream.unwrap();

        println!("connection established");

        let own_pri = Path::new("/home/yamato/.ssh/id_blog_host.pem");
        let dst_pub = Path::new("/home/yamato/.ssh/id_blog.pem.pub");

        let mut msg = Vec::new();
        stream.read_msg(&mut msg).unwrap();

        if let Err(e) = util::auth(&mut stream, &own_pri, &dst_pub) {
            eprintln!("authentication failed");
            panic!("{:?}", e);
        }

        let aes_key;
        match util::exchange_aes_key(&mut stream, &own_pri, &dst_pub) {
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
