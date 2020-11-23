use deployer::tcp_wrap::*;
use std::fs;
use std::io::*;
use std::net::*;
use std::path::Path;
use std::time::Duration;
mod depcl;

fn main() {
    let (section, host, port, mut key, dir) = depcl::get_config();
    let dir = Path::new(&dir);
    let addr = format!("{}:{}", host, port);
    let addr = addr.to_socket_addrs();

    if let Err(e) = addr {
        eprintln!("can't resolve host");
        eprintln!("{:?}", e);
        return;
    }

    let addr = addr.unwrap().find(|x| (*x).is_ipv4()).unwrap();
    match TcpStream::connect_timeout(&addr, Duration::from_millis(2000)) {
        Err(e) => {
            panic!("{:?}", e);
        }
        Ok(mut stream) => {
            stream
                .set_read_timeout(Some(Duration::from_millis(3000)))
                .unwrap();
            stream
                .set_write_timeout(Some(Duration::from_millis(3000)))
                .unwrap();

            stream.write(section.as_bytes()).unwrap();
            stream.flush().unwrap();

            /*
            if let Err(e) = depcl::auth(&stream, &key) {
                panic!("{:?}", e);
            }
            */

            /*
            if let Err(e) = auth() {
                eprintln!("{:?}", e);
                return Err(e);
            }
            */

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
