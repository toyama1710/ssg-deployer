mod dephs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::*;

fn main() {
    let p = dephs::get_config();
    let addr = format!("0.0.0.0:{}", p);
    let listner = TcpListener::bind(addr).unwrap();

    for stream in listner.incoming() {
        let stream = stream.unwrap();
        let mut reader = BufReader::new(&stream);
        let mut writer = BufWriter::new(&stream);
        let mut msg = String::new();

        reader.read_line(&mut msg).unwrap();
        msg = msg.trim().to_owned();
        println!("connection established");
        println!("{}", msg);
        writer.write(format!("{} a\n", msg).as_bytes()).unwrap();
        writer.write(format!("{} b\n", msg).as_bytes()).unwrap();
        writer.flush().unwrap();
    }
}
