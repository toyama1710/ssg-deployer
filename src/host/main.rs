mod dephs;
use deployer::tcp_wrap::*;
use deployer::util;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::*;

fn main() {
    let p = dephs::get_config();
    let addr = format!("0.0.0.0:{}", p);
    let listner = TcpListener::bind(addr).unwrap();

    for stream in listner.incoming() {
        let mut stream = stream.unwrap();

        println!("connection established");

        let mut own_pub = File::open("/home/yamato/.ssh/id_blog_host.pem.pub").unwrap();
        let mut own_pri = File::open("/home/yamato/.ssh/id_blog_host.pem").unwrap();
        let mut dst_pri = File::open("/home/yamato/.ssh/id_blog.pem").unwrap();

        let mut msg = Vec::new();
        stream.read_msg(&mut msg).unwrap();

        if let Err(e) = util::auth(&mut stream, &mut own_pub, &mut own_pri, &mut dst_pri) {
            println!("{:?}", e);
            continue;
        }
    }
}
