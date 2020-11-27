use crate::util;
use openssl::symm::*;
use std::io::{self, Read, Write};
use std::net::TcpStream;

pub trait PlainRW {
    fn read_msg(&mut self, block: &mut Vec<u8>) -> io::Result<usize>;
    fn write_msg(&mut self, block: &Vec<u8>) -> io::Result<usize>;
}

impl PlainRW for TcpStream {
    fn write_msg(&mut self, block: &Vec<u8>) -> std::result::Result<usize, std::io::Error> {
        let mut sz = block.len() as u64;
        let mut buf = [0u8; 8];

        for i in (0..8).rev() {
            buf[i] = (sz & 0xff) as u8;
            sz >>= 8;
        }

        self.write(&buf)?;
        self.write(block.as_slice())?;

        return Ok(block.len());
    }
    fn read_msg(
        &mut self,
        block: &mut std::vec::Vec<u8>,
    ) -> std::result::Result<usize, std::io::Error> {
        let mut sz = 0u64;
        for _ in 0..8 {
            let mut buf = [0u8; 1];
            self.read_exact(&mut buf)?;
            sz <<= 8;
            sz |= buf[0] as u64;
        }
        block.clear();
        block.resize(sz as usize, 0);
        self.read_exact(block.as_mut_slice())?;

        return Ok(block.len());
    }
}

pub trait Aes256cbcWrap {
    fn read_aes(&mut self, key: &[u8], block: &mut Vec<u8>) -> io::Result<usize>;
    fn write_aes(&mut self, key: &[u8], block: &[u8]) -> io::Result<usize>;
}

impl Aes256cbcWrap for TcpStream {
    fn read_aes(&mut self, key: &[u8], block: &mut Vec<u8>) -> io::Result<usize> {
        let iv_len = Cipher::aes_256_cbc().iv_len().unwrap();

        let mut buf = vec![0u8; 32];
        self.read_exact(&mut buf)?;
        let buf = decrypt(Cipher::aes_256_cbc(), key, Some(&vec![0; iv_len]), &buf).unwrap();
        let buf = buf.split_at(iv_len).1;
        let mut sz = 0usize;
        for v in buf {
            sz <<= 8;
            sz |= *v as usize;
        }

        let mut buf = vec![0u8; sz];
        self.read_exact(&mut buf)?;
        let buf = decrypt(Cipher::aes_256_cbc(), key, Some(&vec![0; iv_len]), &buf).unwrap();

        *block = Vec::from(buf.split_at(iv_len).1);

        return Ok(block.len());
    }
    fn write_aes(&mut self, key: &[u8], block: &[u8]) -> io::Result<usize> {
        let iv_len = Cipher::aes_256_cbc().iv_len().unwrap();
        let mut block = Vec::from(block);
        block.reverse();
        block.append(&mut vec![0; iv_len]);
        block.reverse();
        let iv = util::get_rnd_vec(iv_len);
        let msg = encrypt(Cipher::aes_256_cbc(), key, Some(&iv), &block).unwrap();

        let mut sz = msg.len() as u64;
        let mut head = Vec::new();
        for _ in 0..8 {
            head.push((sz & 0xff) as u8);
            sz >>= 8;
        }
        head.append(&mut vec![0; iv_len]);
        head.reverse();

        let head = encrypt(Cipher::aes_256_cbc(), key, Some(&iv), &head).unwrap();

        self.write(head.as_slice())?;
        self.write(msg.as_slice())?;
        return Ok(msg.len());
    }
}
