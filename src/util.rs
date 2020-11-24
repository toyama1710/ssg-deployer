use crate::tcp_wrap::*;
use openssl::rsa::{Padding, Rsa};
use std::fs::File;
use std::io::{self, Read, Write};
use std::io::{Error, ErrorKind};
use std::net::TcpStream;

pub fn get_rnd_vec(s: usize) -> Vec<u8> {
    return (0..s).map(|_| rand::random()).collect();
}
pub fn auth(
    stream: &mut TcpStream,
    own_pub: &mut File,
    own_pri: &mut File,
    dst_pri: &mut File,
) -> io::Result<()> {
    let mut buf = Vec::new();

    own_pub.read_to_end(&mut buf)?;
    let own_pub = Rsa::public_key_from_pem(buf.as_slice())?;

    buf.clear();
    own_pri.read_to_end(&mut buf)?;
    let own_pri = Rsa::private_key_from_pem(buf.as_slice())?;

    buf.clear();
    dst_pri.read_exact(&mut buf)?;
    let dst_pri = Rsa::private_key_from_pem(buf.as_slice())?;

    println!("check");
    let n_slice = own_pub.n().to_vec();
    let e_slice = own_pub.e().to_vec();
    stream.write_msg(&n_slice)?;
    stream.write_msg(&e_slice)?;
    stream.flush()?;

    let mut n = Vec::new();
    stream.read_msg(&mut n)?;
    let mut e = Vec::new();
    stream.read_msg(&mut e)?;
    let dst_pub = Rsa::from_public_components(
        openssl::bn::BigNum::from_slice(n.as_slice())?,
        openssl::bn::BigNum::from_slice(e.as_slice())?,
    )?;

    let org_rnd = get_rnd_vec(1024);
    let mut snd = [0u8; 1024];
    own_pub.public_encrypt(&org_rnd, &mut snd, Padding::PKCS1)?;
    stream.write(&snd)?;
    stream.flush()?;

    let mut msg = [0u8; 1024];
    let mut tmp = [0u8; 1024];
    let mut snd = [0u8; 1024];
    stream.read_exact(&mut msg)?;
    dst_pri.private_decrypt(&msg, &mut tmp, Padding::PKCS1)?;
    dst_pub.public_encrypt(&tmp, &mut snd, Padding::PKCS1)?;
    stream.write(&snd)?;
    stream.flush()?;

    let mut msg = [0u8; 1024];
    let mut tmp = [0u8; 1024];
    stream.read_exact(&mut msg)?;
    own_pri.private_decrypt(&msg, &mut tmp, Padding::PKCS1)?;
    for i in 0..1024 {
        if tmp[i] != org_rnd[i] {
            stream.write(&[1u8; 1])?;
            return Err(Error::new(ErrorKind::Other, "Authentication failed"));
        }
    }

    stream.write(&[0u8; 1])?;

    let mut msg = [0u8; 1];
    stream.read(&mut msg)?;
    if msg[0] != 0 {
        return Err(Error::new(ErrorKind::Other, "Authentication failed"));
    }

    return Ok(());
}
