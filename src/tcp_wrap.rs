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

pub trait AesWrap {
    fn read_aes(&mut self, key: &[u8], block: &mut Vec<u8>) -> io::Result<usize>;
    fn write_aes(&mut self, key: &[u8], iv: &[u8], block: &mut Vec<u8>) -> io::Result<usize>;
}

impl AesWrap for TcpStream {
    fn read_aes(&mut self, key: &[u8], block: &mut Vec<u8>) -> io::Result<usize> {
        let mut buf = vec![0u8; 32];
        self.read_exact(buf.as_mut_slice())?;
        let buf = decrypt(Cipher::aes_256_cbc(), key, Some(&[0; 16]), buf.as_slice()).unwrap();
        let buf = buf.split_at(16).1;
        let mut sz = 0usize;
        for v in buf {
            sz <<= 8;
            sz |= *v as usize;
        }

        let mut buf = vec![0u8; sz + 16];
        self.read_exact(buf.as_mut_slice())?;
        let buf = decrypt(Cipher::aes_256_cbc(), key, Some(&[0; 16]), buf.as_slice()).unwrap();
        block.clear();
        block.resize(sz, 0);

        for i in 16..sz {
            block[i - 16] = buf[i];
        }

        return Ok(sz);
    }
    fn write_aes(&mut self, key: &[u8], iv: &[u8], block: &mut Vec<u8>) -> io::Result<usize> {
        block.reverse();
        block.append(&mut vec![0; 16]);
        block.reverse();
        let msg = encrypt(Cipher::aes_256_cbc(), key, Some(iv), block.as_slice()).unwrap();

        let mut sz = msg.len() as u64;
        let mut head = Vec::new();
        for _ in 0..8 {
            head.push((sz & 0xff) as u8);
            sz >>= 8;
        }
        head.append(&mut vec![0; 16]);
        head.reverse();

        let head = encrypt(Cipher::aes_256_cbc(), key, Some(iv), head.as_slice()).unwrap();

        self.write(head.as_slice())?;
        self.write(msg.as_slice())?;
        return Ok(msg.len());
    }
}
