use clap::{self, App, Arg};
use deployer::tcp_wrap;
use openssl::rsa::{Padding, Rsa};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::io::{Error, ErrorKind};
use std::net::TcpStream;
use std::path::Path;

// return (section, hostname, port, keyfile, publish_dir)
pub fn get_config() -> (String, String, u32, File, String) {
    let args = App::new("ssg-deployer(client)")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about("simple deployer")
        .args(&[
            Arg::from_usage("[section]<SECTION>"),
            Arg::from_usage("[hostname] -d <HOST> 'assign destination'"),
            Arg::from_usage("[port] -p --port <INTEGER> 'sets port number to send request'")
                .validator(|s| {
                    let v = s.parse::<usize>();
                    match v {
                        Ok(v) => {
                            if 1024 <= v && v <= 49512 {
                                return Ok(());
                            } else {
                                return Err(String::from("port number must be 1024..=49512"));
                            }
                        }
                        Err(_) => {
                            return Err(String::from("-p value must be integer"));
                        }
                    }
                }),
            Arg::from_usage("[identity file] -i --identity <FILE> 'sets identity file to use'")
                .validator(|s| {
                    let p = Path::new(std::ffi::OsStr::new(&s));
                    if p.exists() && p.is_file() {
                        Ok(())
                    } else {
                        Err(String::from("file not found"))
                    }
                }),
            Arg::from_usage("[publish_dir] -s <PUBLISH_DIR> 'assign publish directory").validator(
                |s| {
                    let p = Path::new(std::ffi::OsStr::new(&s));
                    if p.exists() && p.is_dir() {
                        Ok(())
                    } else {
                        Err(String::from("missing path"))
                    }
                },
            ),
        ])
        .get_matches();

    let section = String::from(args.value_of("section").unwrap());
    let host = String::from(args.value_of("hostname").unwrap());
    let port = args.value_of("port").unwrap().parse::<u32>().unwrap();
    let path = args
        .value_of("identity file")
        .unwrap()
        .to_owned()
        .to_string();
    let key = File::open(path).unwrap();
    let dir = String::from(args.value_of("publish_dir").unwrap());
    return (section, host, port, key, dir);
}

pub fn auth(
    stream: &mut TcpStream,
    cl_pub: &mut File,
    cl_pri: &mut File,
    hs_pri: &mut File,
) -> io::Result<()> {
    let mut buf = Vec::new();

    cl_pub.read_to_end(&mut buf)?;
    let cl_pub = Rsa::public_key_from_pem(buf.as_slice())?;

    buf.clear();
    cl_pri.read_to_end(&mut buf)?;
    let cl_pri = Rsa::private_key_from_pem(buf.as_slice())?;

    buf.clear();
    hs_pri.read_exact(&mut buf)?;
    let hs_pri = Rsa::private_key_from_pem(buf.as_slice())?;

    let mut sz = [0u8; 2];
    stream.read_exact(&mut sz)?;
    let sz = sz[0] << 8 + sz[1];
    let mut n = vec![0u8; sz.into()];
    stream.read_exact(n.as_mut_slice())?;

    let mut sz = [0u8; 2];
    stream.read_exact(&mut sz)?;
    let sz = sz[0] << 8 + sz[1];
    let mut e = vec![0u8; sz.into()];
    stream.read_exact(e.as_mut_slice())?;

    let hs_pub = Rsa::from_public_components(
        openssl::bn::BigNum::from_slice(n.as_slice())?,
        openssl::bn::BigNum::from_slice(e.as_slice())?,
    )?;

    // send cl_pub

    // host -> client
    let mut rcv = [0u8; 256];
    let mut tmp = [0u8; 256];
    let mut snd = [0u8; 256];
    stream.read_exact(&mut rcv)?;
    hs_pri.private_decrypt(&rcv, &mut tmp, Padding::PKCS1)?;
    cl_pub.public_encrypt(&rcv, &mut snd, Padding::PKCS1)?;
    stream.write(&snd)?;
    stream.flush()?;

    let mut tmp = [0u8; 1];
    stream.read_exact(&mut tmp)?;
    if tmp[0] != 0 {
        return Err(Error::new(ErrorKind::Other, "Authentication failed"));
    }

    // client -> host
    let mut snd = get_rnd(256);
    let mut tmp = [0u8; 256];
    cl_pub.public_encrypt(&snd, &mut tmp, Padding::PKCS1);
    stream.write(&tmp)?;
    stream.flush()?;
    stream.read_exact(&mut rcv);
    hs_pri.private_decrypt(&rcv, &mut tmp, Padding::PKCS1)?;
    for i in 0..256 {
        if tmp[i] != snd[i] {
            return Err(Error::new(ErrorKind::Other, "Authentication failed"));
        }
    }

    return Ok(());
}

pub fn visit_dir(dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dir(&path)?;
            } else {
                println!("{}", path.display());
            }
        }
    }
    Ok(())
}
