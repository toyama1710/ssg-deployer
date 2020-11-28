use crate::tcp_wrap::Aes256cbcWrap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::io::{Error, ErrorKind};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

pub fn send_file(stream: &mut TcpStream, aes_key: &[u8], pub_dir: &Path) -> io::Result<()> {
    let f = |stream: &mut TcpStream| {
        let mut buf = Vec::new();
        stream.read_aes(&aes_key, &mut buf).unwrap();
        return String::from_utf8(buf).unwrap();
    };

    loop {
        let s = f(stream);
        if s == ";end" {
            break;
        }

        let path = pub_dir.join(&s);
        if path.is_file() {
            let mut file;
            match File::open(path) {
                Ok(f) => {
                    file = f;
                    stream.write_aes(&aes_key, b";ok")?;
                }
                Err(_) => {
                    stream.write_aes(&aes_key, b";err file found but cant open")?;
                    continue;
                }
            }
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            stream.write_aes(&aes_key, &buf)?;
            stream.flush()?;
        } else {
            stream.write_aes(&aes_key, b";err file not found")?;
        }
    }

    stream.write_aes(&aes_key, b";end")?;
    stream.flush()?;
    return Ok(());
}

pub fn rcv_file(
    stream: &mut TcpStream,
    aes_key: &[u8],
    pub_dir: &Path,
    req: &[PathBuf],
) -> io::Result<usize> {
    let mut cnt = 0;
    let f = |stream: &mut TcpStream| {
        let mut buf = Vec::new();
        stream.read_aes(&aes_key, &mut buf).unwrap();
        return String::from_utf8(buf).unwrap();
    };

    for path in req {
        let s = f(stream);
        if s == ";end" {
            return Err(Error::new(ErrorKind::Other, "req has too many item"));
        } else if s.starts_with(";err") {
            continue;
        }

        let path = pub_dir.join(path);
        let par_dir = path.with_file_name("");
        if !par_dir.exists() {
            fs::create_dir_all(&par_dir)?;
        }
        let mut file = File::create(&path)?;

        let mut buf = Vec::new();
        stream.read_aes(&aes_key, &mut buf)?;
        file.write(&buf)?;

        cnt += 1;
    }

    return Ok(cnt);
}
