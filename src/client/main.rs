use deployer::file_shake::*;
use deployer::tcp_wrap::*;
use deployer::util;
use std::io::*;
use std::net::*;
use std::time::Duration;
mod depcl;

fn main() {
    let conf = depcl::get_config();
    if let Err(e) = conf {
        eprintln!("{:?}", e);
        panic!();
    }
    let section = conf.unwrap();
    let addr = format!("{}:{}", section.host, section.port);
    let addr = addr.to_socket_addrs();

    if let Err(e) = addr {
        eprintln!("can't resolve host");
        eprintln!("{:?}", e);
        panic!();
    }

    let addr = addr.unwrap().find(|x| (*x).is_ipv4()).unwrap();
    match TcpStream::connect_timeout(&addr, Duration::from_millis(2000)) {
        Err(e) => {
            eprintln!("{:?}", e);
            panic!();
        }
        Ok(mut stream) => {
            stream
                .set_read_timeout(Some(Duration::from_millis(5000)))
                .unwrap();
            stream
                .set_write_timeout(Some(Duration::from_millis(5000)))
                .unwrap();

            stream.write_msg(&section.user.as_bytes()).unwrap();
            stream.flush().unwrap();
            if let Err(e) = util::auth(&mut stream, &section.own_pri, &section.host_pub) {
                eprintln!("authentication failed");
                eprintln!("{:?}", e);
                panic!();
            }

            let aes_key;
            match util::exchange_aes_key(&mut stream, &section.own_pri, &section.host_pub) {
                Err(e) => {
                    eprintln!("failed to exchange AES KEY");
                    eprintln!("{:?}", e);
                    panic!();
                }
                Ok(v) => {
                    aes_key = v;
                }
            }

            stream
                .write_aes(&aes_key, &mut Vec::from(section.name.as_bytes()))
                .unwrap();
            stream.flush().unwrap();

            let hashes = util::calc_hash(&section.publish_dir).unwrap();

            depcl::send_hash(&mut stream, &aes_key, &section.publish_dir, &hashes).unwrap();
            stream.flush().unwrap();

            send_file(&mut stream, &aes_key, &section.publish_dir).unwrap();

            let mut buf = Vec::new();
            stream.read_aes(&aes_key, &mut buf).unwrap();

            if String::from_utf8_lossy(&buf) == ";session end" {
                println!("end");
            } else {
                eprintln!("file sended but some error occured");
                panic!();
            }
        }
    }
}
