use crate::tcp_wrap::*;
use openssl::rsa::{Padding, Rsa};
use std::fs::File;
use std::io::{self, Read, Write};
use std::io::{Error, ErrorKind};
use std::net::TcpStream;
use std::path::Path;

pub fn get_rnd_vec(s: usize) -> Vec<u8> {
    return (0..s).map(|_| rand::random()).collect();
}
pub fn auth(
    stream: &mut TcpStream,
    own_pub: &Path,
    own_pri: &Path,
    dst_pub: &Path,
) -> io::Result<()> {
    const KEY_BIT: usize = 4096;
    const KEY_BYTE: usize = KEY_BIT / 8;
    const ENCLYPT_LEN: usize = KEY_BYTE - 16;

    let mut own_pub = File::open(own_pub)?;
    let mut own_pri = File::open(own_pri)?;
    let mut dst_pub = File::open(dst_pub)?;

    let mut buf = Vec::new();
    own_pub.read_to_end(&mut buf)?;
    let own_pub = Rsa::public_key_from_pem(buf.as_slice())?;

    buf.clear();
    own_pri.read_to_end(&mut buf)?;
    let own_pri = Rsa::private_key_from_pem(buf.as_slice())?;

    buf.clear();
    dst_pub.read_to_end(&mut buf)?;
    let dst_pub = Rsa::public_key_from_pem(buf.as_slice())?;

    assert_eq!(own_pub.size(), KEY_BYTE as u32);
    assert_eq!(own_pri.size(), KEY_BYTE as u32);
    assert_eq!(dst_pub.size(), KEY_BYTE as u32);

    let org_rnd = get_rnd_vec(ENCLYPT_LEN);
    let mut snd = [0u8; KEY_BYTE];
    dst_pub.public_encrypt(org_rnd.as_slice(), &mut snd, Padding::PKCS1)?;
    stream.write(&snd)?;
    stream.flush()?;

    let mut msg = [0u8; KEY_BYTE];
    let mut tmp = [0u8; KEY_BYTE];
    let mut snd = [0u8; KEY_BYTE];
    stream.read_exact(&mut msg)?;
    own_pri.private_decrypt(&msg, &mut tmp, Padding::PKCS1)?;
    let tmp = Vec::from(&tmp[0..ENCLYPT_LEN]);
    dst_pub.public_encrypt(&tmp, &mut snd, Padding::PKCS1)?;
    stream.write(&snd)?;
    stream.flush()?;

    let mut msg = [0u8; KEY_BYTE];
    let mut tmp = [0u8; KEY_BYTE];
    stream.read_exact(&mut msg)?;
    own_pri.private_decrypt(&msg, &mut tmp, Padding::PKCS1)?;
    for i in 0..ENCLYPT_LEN {
        if tmp[i] != org_rnd[i] {
            stream.write(&[1u8; 1])?;
            return Err(Error::new(ErrorKind::Other, "data verify failed"));
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
