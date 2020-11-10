use clap::{App, Arg};
use toml::Value;
use std::fs;

fn main() {
    let args = App::new("ssg-deployer(host)")
    .version("0.0.0")
    .author("toyama1710")
    .about("simple deployer")
    .arg(
        Arg::with_name("identity file")
            .short("i")
            .long("identity")
            .help("sets identity file(.pem) to use")
            .takes_value(true)
            .value_name("FILE")
    )
    .arg(
        Arg::with_name("port")
            .short("p")
            .long("port")
            .help("sets port number to use")
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

    let conf = fs::read_to_string("/etc/ssg-deployer.d/config.toml");
}
