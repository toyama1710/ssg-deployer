mod dephs;
use deployer::file_shake::rcv_file;
use deployer::tcp_wrap::*;
use deployer::util;
use std::collections::BTreeMap;
use std::io::Write;
use std::net::*;
use std::path::PathBuf;

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
        if let Err(_e) = stream {
            continue;
        }

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

        let mut msg = Vec::new();
        stream.read_aes(&aes_key, &mut msg).unwrap();
        let msg = String::from_utf8(msg).unwrap();

        let sec_conf = sections.iter().find(|v| v.name == msg);
        if sec_conf.is_none() {
            eprintln!("section not found");
            continue;
        }
        let sec_conf = sec_conf.unwrap();

        let h = util::calc_hash(&sec_conf.publish_dir).unwrap();
        let mut hashes = BTreeMap::<PathBuf, [u8; 32]>::new();
        for v in h.iter() {
            let p = v.0.strip_prefix(&sec_conf.publish_dir).unwrap();
            hashes.insert(p.into(), v.1);
        }

        let mut req = Vec::new();
        loop {
            let mut msg = Vec::new();
            stream.read_aes(&aes_key, &mut msg).unwrap();
            let msg = String::from_utf8(msg).unwrap();
            if msg.starts_with(";hash sended") {
                break;
            }

            let path = PathBuf::from(&msg);
            let mut msg = Vec::new();
            stream.read_aes(&aes_key, &mut msg).unwrap();
            let msg = util::into_u8_32(&msg);
            if !hashes.contains_key(&path) {
                req.push(path);
            } else {
                if hashes.get(&path).unwrap() != &msg {
                    hashes.remove(&path);
                    req.push(path);
                } else {
                    hashes.remove(&path);
                }
            }
        }

        rcv_file(&mut stream, &aes_key, &sec_conf.publish_dir, &req).unwrap();

        for (path, _) in hashes {
            let path = sec_conf.publish_dir.join(&path);
            std::fs::remove_file(path).unwrap();
        }

        util::clear_dir(&sec_conf.publish_dir).unwrap();
    }
}
