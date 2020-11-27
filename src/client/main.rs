use deployer::tcp_wrap::*;
use deployer::util;
use std::io::*;
use std::net::*;
use std::time::Duration;
mod depcl;

fn main() {
    let section = depcl::get_config().unwrap();
    let addr = format!("{}:{}", section.hostname, section.port);
    let addr = addr.to_socket_addrs();

    let own_pri = section.own_pri;
    let dst_pub = section.host_pub;

    if let Err(e) = addr {
        eprintln!("can't resolve host");
        panic!("{:?}", e);
    }

    let addr = addr.unwrap().find(|x| (*x).is_ipv4()).unwrap();
    match TcpStream::connect_timeout(&addr, Duration::from_millis(2000)) {
        Err(e) => {
            panic!("{:?}", e);
        }
        Ok(mut stream) => {
            stream
                .set_read_timeout(Some(Duration::from_millis(5000)))
                .unwrap();
            stream
                .set_write_timeout(Some(Duration::from_millis(5000)))
                .unwrap();

            stream
                .write_msg(&Vec::from(section.name.as_bytes()))
                .unwrap();
            stream.flush().unwrap();

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

            // send_hashs()
            /*
                for entry in get_hashs() {
                }
            */

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
