use std::io;
use std::fs;
use std::path::Path;
use clap::{App, Arg};
use std::fs::File;

// return (hostname, port, keyfile)
pub fn get_config() -> (String, u32, File) {
    let args = App::new("ssg-deployer(client)")
    .version("0.0.0")
    .author("toyama1710")
    .about("simple deployer")
    .arg(
        Arg::with_name("hostname")
             .short("d")
             .help("assign destination")
             .takes_value(true)
             .value_name("host")
    )
    .arg(
        Arg::with_name("identity file")
            .short("i")
            .long("identity")
            .help("sets identity file to use")
            .takes_value(true)
            .value_name("FILE")
            .validator(|s| {
                let p = Path::new(std::ffi::OsStr::new(&s));
                if p.exists() { Ok(()) }
                else { Err(String::from("file not found")) }
            })
    )
    .arg(
        Arg::with_name("port")
            .short("p")
            .long("port")
            .help("sets port number to send request")
            .takes_value(true)
            .value_name("INTEGER")
            .validator(|s| {
                let v = s.parse::<usize>();
                match v {
                    Ok(v) => {
                        if 1024 <= v && v <= 49512 { return Ok(()); }
                        else { return Err(String::from("port number must be 1024..=49512")); }
                    },
                    Err(_) => {
                        return Err(String::from("-p value must be integer"));
                    },
                }
            })
    )
    .get_matches();

    let host = args.value_of("hostname").unwrap().to_owned().to_string();
    let p = args.value_of("port").unwrap().parse::<u32>().unwrap();
    let path = args.value_of("identity file").unwrap().to_owned().to_string();
    let key = File::open(path).unwrap();
    return (host, p, key);
}

fn visit_dir(dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dir(&path)?;
            }
            else {
                println!("{}", path.display());
            }
        }
    }
    Ok(())
}

fn main() {
    let (host, port, mut key) = get_config();
}
