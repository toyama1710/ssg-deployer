use openssl::pkey::{Private, Public};
use openssl::rsa::{Padding, Rsa};
use openssl::sha::sha256;
use openssl::symm::*;
use rand::Rng;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::io::{Error, ErrorKind};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

pub fn get_rnd_vec(s: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    return (0..s).map(|_| rng.gen()).collect();
}

const RSA_KEY_BIT: usize = 4096;
const RSA_KEY_BYTE: usize = RSA_KEY_BIT / 8;
const RSA_PLAIN_MAX: usize = RSA_KEY_BYTE - 16;
// authentication succeed -> return aes_key
//                failed  -> retrun Err
pub fn auth(stream: &mut TcpStream, own_pri: &Path, dst_pub: &Path) -> io::Result<()> {
    let (own_pri, dst_pub) = construct_rsa_key(own_pri, dst_pub)?;

    let org_rnd = get_rnd_vec(RSA_PLAIN_MAX);
    let mut snd = [0u8; RSA_KEY_BYTE];
    dst_pub.public_encrypt(&org_rnd, &mut snd, Padding::PKCS1)?;
    stream.write(&snd)?;
    stream.flush()?;

    // reply
    let mut msg = [0u8; RSA_KEY_BYTE];
    let mut tmp = [0u8; RSA_KEY_BYTE];
    let mut snd = [0u8; RSA_KEY_BYTE];
    stream.read_exact(&mut msg)?;
    own_pri.private_decrypt(&msg, &mut tmp, Padding::PKCS1)?;
    let tmp = Vec::from(&tmp[0..RSA_PLAIN_MAX]);
    dst_pub.public_encrypt(&tmp, &mut snd, Padding::PKCS1)?;
    stream.write(&snd)?;
    stream.flush()?;

    // recieve reply
    let mut msg = [0u8; RSA_KEY_BYTE];
    let mut tmp = [0u8; RSA_KEY_BYTE];
    stream.read_exact(&mut msg)?;
    own_pri.private_decrypt(&msg, &mut tmp, Padding::PKCS1)?;

    // verify data
    for i in 0..RSA_PLAIN_MAX {
        if tmp[i] != org_rnd[i] {
            // code: fail
            stream.write(&[1u8; 1])?;
            stream.flush()?;
            return Err(Error::new(ErrorKind::Other, "data verify failed"));
        }
    }

    // code: success
    stream.write(&[0u8; 1])?;
    stream.flush()?;

    return Ok(());
}

fn construct_rsa_key(own_pri: &Path, dst_pub: &Path) -> io::Result<(Rsa<Private>, Rsa<Public>)> {
    let mut own_pri = File::open(own_pri)?;
    let mut dst_pub = File::open(dst_pub)?;

    let mut buf = Vec::new();
    own_pri.read_to_end(&mut buf)?;
    let own_pri = Rsa::private_key_from_pem(buf.as_slice())?;

    let mut buf = Vec::new();
    dst_pub.read_to_end(&mut buf)?;
    let dst_pub = Rsa::public_key_from_pem(buf.as_slice())?;

    assert_eq!(own_pri.size(), RSA_KEY_BYTE as u32);
    assert_eq!(dst_pub.size(), RSA_KEY_BYTE as u32);

    return Ok((own_pri, dst_pub));
}

pub fn exchange_aes_key(
    stream: &mut TcpStream,
    own_pri: &Path,
    dst_pub: &Path,
) -> io::Result<Vec<u8>> {
    let (own_pri, dst_pub) = construct_rsa_key(own_pri, dst_pub)?;
    let mut msg = [0u8; 1];
    stream.read(&mut msg)?;
    if msg[0] != 0 {
        return Err(Error::new(ErrorKind::Other, "Authentication failed"));
    }

    let snd_aes_key = get_rnd_vec(Cipher::aes_256_cbc().key_len());
    let mut msg = [0u8; RSA_KEY_BYTE];
    dst_pub.public_encrypt(&snd_aes_key, &mut msg, Padding::PKCS1)?;
    stream.write(&msg)?;
    stream.flush()?;

    let mut rcv_aes_key = [0u8; RSA_KEY_BYTE];
    stream.read_exact(&mut msg)?;
    own_pri.private_decrypt(&msg, &mut rcv_aes_key, Padding::PKCS1)?;

    let rcv_aes_key = Vec::from(&rcv_aes_key[0..Cipher::aes_256_cbc().key_len()]);

    let aes_key: Vec<u8> = (0..Cipher::aes_256_cbc().key_len())
        .map(|i| snd_aes_key[i] ^ rcv_aes_key[i])
        .collect();

    return Ok(aes_key);
}

pub fn calc_hash(path: &Path) -> io::Result<Vec<(PathBuf, [u8; 32])>> {
    let mut ret = Vec::new();
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            ret.append(&mut calc_hash(&path)?);
        }
        return Ok(ret);
    } else {
        let mut buf = Vec::new();
        let mut file = File::open(path)?;
        file.read_to_end(&mut buf)?;
        ret.push((path.to_path_buf(), sha256(&buf)));
        return Ok(ret);
    }
}
