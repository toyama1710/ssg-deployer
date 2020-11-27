use clap::{self, App, Arg};
use serde::{Deserialize, Serialize};
use std::convert::From;
use std::fs::File;
use std::io::{self, Error, ErrorKind, Read};
use std::path::{Component, PathBuf};

#[derive(Deserialize, Serialize)]
pub struct SectionTable {
    pub name: String,
    pub publish_dir: PathBuf,
    pub client_pub: PathBuf,
    pub own_pri: PathBuf,
}

#[derive(Deserialize, Serialize)]
struct ConfigToml {
    port: Option<u32>,
    section: Vec<SectionTable>,
}

// return listen port
pub fn get_config() -> io::Result<(u32, Vec<SectionTable>)> {
    let mut conf_path = PathBuf::from(Component::RootDir.as_os_str());
    conf_path.push("etc");
    conf_path.push("ssg-deployer");
    conf_path.push("config.toml");
    let conf_path: PathBuf = conf_path.iter().collect();

    let args = App::new("ssg-deployer(host)")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about("simple deployer")
        .args(&[
            Arg::from_usage("[port] -p --port <integer> 'sets port to listen'").validator(|s| {
                let v = s.parse::<u32>();
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
        ])
        .after_help(&*format!("config file is {:?}", conf_path))
        .get_matches();

    let mut buf = Vec::new();
    let mut conf_file = File::open(conf_path)?;
    conf_file.read_to_end(&mut buf)?;
    let conf_toml: ConfigToml = toml::from_slice(&buf)?;

    let mut p = None;
    // port validation
    if let Some(v) = args.value_of("port") {
        p = Some(v.parse::<u32>().unwrap());
    } else if let Some(v) = conf_toml.port {
        p = Some(v);
    }
    if p.is_none() {
        return Err(Error::new(ErrorKind::Other, "port number is not satisfied"));
    } else if !(1024..=49512).contains(&p.unwrap()) {
        return Err(Error::new(
            ErrorKind::Other,
            "port number must be 1024..=49512",
        ));
    }

    // conf_toml validation
    for item in &conf_toml.section {
        if !item.client_pub.is_file() {
            return Err(Error::new(ErrorKind::NotFound, "client_pub is not exists"));
        }
        if !item.own_pri.is_file() {
            return Err(Error::new(ErrorKind::NotFound, "own_pub is not exists"));
        }
    }

    for t in &conf_toml.section {
        let mut cnt = 0;
        for v in &conf_toml.section {
            if t.name == v.name {
                cnt += 1;
            }
        }

        if cnt > 1 {
            return Err(Error::new(
                ErrorKind::Other,
                format!("{} found more than one", t.name),
            ));
        }
    }

    return Ok((p.unwrap(), conf_toml.section));
}
