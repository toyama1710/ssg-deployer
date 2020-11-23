use openssl::symm::*;
use std::io::{self, Read, Write};
use std::net::TcpStream;

pub trait ReadTcp {
    // buf.back() == ch
    fn read_until(&mut self, ch: u8, buf: &mut Vec<u8>) -> io::Result<usize>;
}

impl ReadTcp for TcpStream {
    fn read_until(&mut self, ch: u8, buf: &mut std::vec::Vec<u8>) -> io::Result<usize> {
        buf.clear();
        let mut byte = [0u8; 1];

        loop {
            self.read_exact(&mut byte)?;
            buf.push(byte[0]);
            if byte[0] == ch {
                break;
            }
        }

        return Ok(buf.len());
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
        self.flush()?;
        return Ok(msg.len());
    }
}
