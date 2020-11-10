use std::io::*;
use std::fs;
use std::path::Path;
use std::net::*;
mod depcl;

fn main() {
    let (section, host, port, mut key, dir) = depcl::get_config();
    let dir = Path::new(&dir);
    let addr = format!("{}:{}", host, port);
    let addr = addr.to_socket_addrs();
    if let Err(_) = addr { 
        eprintln!("can't resolve host");
        return;
    }

    let addr = addr.unwrap().find(|x| (*x).is_ipv4()).unwrap();
    match TcpStream::connect(addr) {
        Err(e) => {
            eprintln!("{:?}", e);
            return;
        }
        Ok(stream) => {
            let mut reader = BufReader::new(&stream);
            let mut writer = BufWriter::new(&stream);
            let mut msg = String::new();

            writer.write(section.as_bytes()).unwrap();

            // auth
            /*
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