use deployer::tcp_wrap::Aes256cbcWrap;
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
    let addr = format!("{}:{}", section.hostname, section.port);
    let addr = addr.to_socket_addrs();

    println!(
        "{} {}",
        section.host_pub.display(),
        section.own_pri.display()
    );

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

            let hashes = depcl::calc_hash(&section.publish_dir).unwrap();

            for v in hashes.iter() {
                let p = v.0.strip_prefix(&section.publish_dir).unwrap();
                println!("{}", p.display());
                stream
                    .write_aes(&aes_key, p.to_str().unwrap().as_bytes())
                    .unwrap();
                stream.write_aes(&aes_key, &mut Vec::from(v.1)).unwrap();
            }

            stream.write_aes(&aes_key, b";sended").unwrap();
            stream.flush().unwrap();

            // send_files()
            /*
                loop {
                    reader.read_line(&mut msg);
                    if msg != "end." {
                        send_file();
                    }
                }
            */

            /*
                writer.write("all file sended\n")
            */
        }
    }
}
