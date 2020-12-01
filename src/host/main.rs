mod dephs;
use dephs::{AuthTable, SectionTable};
use deployer::file_shake::rcv_file;
use deployer::tcp_wrap::*;
use deployer::util;
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::{self, Error, ErrorKind};
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
        match stream {
            Err(e) => {
                let mut file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open("/etc/ssg-deployer/error.log")
                    .unwrap();
                file.write(format!("{}", e).as_bytes()).unwrap();
                continue;
            }
            Ok(mut stream) => {
                if let Err(e) = process(&mut stream, &auths, &sections) {
                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open("/etc/ssg-deployer/error.log")
                        .unwrap();
                    file.write(format!("{}", e).as_bytes()).unwrap();
                    continue;
                }
            }
        }
    }
}

fn process(
    stream: &mut TcpStream,
    auths: &[AuthTable],
    sections: &[SectionTable],
) -> io::Result<()> {
    let mut msg = Vec::new();
    stream.read_msg(&mut msg)?;
    let msg = String::from_utf8(msg).unwrap();

    let auth_conf = auths
        .iter()
        .find(|v| v.user == msg)
        .ok_or(Error::new(ErrorKind::NotFound, "user not found"))?;

    util::auth(stream, &auth_conf.own_pri, &auth_conf.client_pub)?;

    let aes_key = util::exchange_aes_key(stream, &auth_conf.own_pri, &auth_conf.client_pub)?;

    let mut msg = Vec::new();
    stream.read_aes(&aes_key, &mut msg)?;
    let msg = String::from_utf8(msg).unwrap();

    let sec_conf = sections
        .iter()
        .find(|v| v.name == msg)
        .ok_or(Error::new(ErrorKind::NotFound, "section not found"))?;

    let h = util::calc_hash(&sec_conf.publish_dir)?;
    let mut hashes = BTreeMap::<PathBuf, [u8; 32]>::new();
    for v in h.iter() {
        let p = v.0.strip_prefix(&sec_conf.publish_dir).unwrap();
        hashes.insert(p.into(), v.1);
    }

    let mut req = Vec::new();
    loop {
        let mut msg = Vec::new();
        stream.read_aes(&aes_key, &mut msg)?;
        let msg = String::from_utf8(msg).unwrap();
        if msg.starts_with(";hash sended") {
            break;
        }

        let path = PathBuf::from(&msg);
        let mut msg = Vec::new();
        stream.read_aes(&aes_key, &mut msg)?;
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

    rcv_file(stream, &aes_key, &sec_conf.publish_dir, &req).unwrap();

    for (path, _) in hashes {
        let path = sec_conf.publish_dir.join(&path);
        std::fs::remove_file(path)?;
    }

    util::clear_empty_dir(&sec_conf.publish_dir)?;

    return Ok(());
}
