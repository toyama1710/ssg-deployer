use clap::{self, App, Arg};
use std::path::Path;

// return (section, hostname, port, keyfile, publish_dir)
pub fn get_config() -> (String, String, u32, String) {
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
    let dir = String::from(args.value_of("publish_dir").unwrap());
    return (section, host, port, dir);
}

/*
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

*/
