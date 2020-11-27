mod dephs;
use deployer::tcp_wrap::*;
use deployer::util;
use std::net::*;

fn main() {
    let conf = dephs::get_config();
    if let Err(e) = conf {
        eprintln!("{:?}", e);
        panic!();
    }
    let (p, sections, auths) = conf.unwrap();

    let addr = format!("0.0.0.0:{}", p);
    let listner = TcpListener::bind(addr).unwrap();

    for stream in listner.incoming() {
        let mut stream = stream.unwrap();

        println!("connection established");

        let mut msg = Vec::new();
        stream.read_msg(&mut msg).unwrap();
        let msg = String::from_utf8(msg).unwrap();

        let auth_conf = auths.iter().find(|v| v.user == msg);
        if auth_conf.is_none() {
            eprintln!("access from not registered user");
            continue;
        }
        let auth_conf = auth_conf.unwrap();

        if let Err(e) = util::auth(&mut stream, &auth_conf.own_pri, &auth_conf.client_pub) {
            eprintln!("authentication failed");
            eprintln!("{:?}", e);
            continue;
        }

        let aes_key;
        match util::exchange_aes_key(&mut stream, &auth_conf.own_pri, &auth_conf.client_pub) {
            Err(e) => {
                eprintln!("failed to exchange AES key");
                eprintln!("{:?}", e);
                continue;
            }
            Ok(v) => {
                aes_key = v;
            }
        }

        let mut sec_name = Vec::new();
        stream.read_aes(&aes_key, &mut sec_name).unwrap();
        let sec_name = String::from_utf8_lossy(&sec_name);

        println!("{}", sec_name);
    }
}
