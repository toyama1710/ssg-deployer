use clap::{self, App, Arg};
use deployer::tcp_wrap::Aes256cbcWrap;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Error, ErrorKind, Read};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub name: String,
    pub user: String,
    pub host: String,
    pub port: u32,
    pub publish_dir: PathBuf,
    pub own_pri: PathBuf,
    pub host_pub: PathBuf,
}
type SectionTable = Config;

#[derive(Deserialize, Serialize, Debug)]
struct ConfigArray {
    section: Vec<SectionTable>,
}

// return (section, hostname, port, keyfile, publish_dir)
pub fn get_config() -> io::Result<Config> {
    let mut conf_path = dirs::config_dir().unwrap();
    conf_path.push("ssg-deployer");
    conf_path.push("config.toml");
    let conf_path: PathBuf = conf_path.iter().collect();

    let args = App::new("ssg-deployer(client)")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about("simple deployer")
        .args(&[Arg::from_usage("<section> <SECTION>")])
        .after_help(&*format!("config file is {:?}", conf_path))
        .get_matches();

    let section = String::from(args.value_of("section").unwrap());

    let mut buf = Vec::new();
    let mut conf_file = File::open(conf_path)?;
    conf_file.read_to_end(&mut buf)?;
    let conf_array: ConfigArray = toml::from_slice(&buf)?;
    let conf_array: Vec<Config> = conf_array
        .section
        .iter()
        .filter(|&v| if *v.name == section { true } else { false })
        .cloned()
        .collect();

    if conf_array.len() > 1 {
        return Err(Error::new(
            ErrorKind::Other,
            "specified section are found, but there are more than one",
        ));
    } else if conf_array.is_empty() {
        return Err(Error::new(
            ErrorKind::Other,
            "specified section is not found",
        ));
    }

    return Ok(conf_array[0].clone());
}

pub fn send_hash(
    stream: &mut TcpStream,
    aes_key: &[u8],
    pre: &Path,
    hashes: &Vec<(PathBuf, [u8; 32])>,
) -> io::Result<()> {
    for v in hashes.iter() {
        let p = v.0.strip_prefix(&pre).unwrap();
        stream.write_aes(&aes_key, p.to_str().unwrap().as_bytes())?;
        stream.write_aes(&aes_key, &mut Vec::from(v.1))?;
    }

    stream.write_aes(&aes_key, b";send").unwrap();

    return Ok(());
}
