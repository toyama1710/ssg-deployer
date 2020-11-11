use clap::{App, Arg};
use toml::Value;
use std::fs;

// return listen port
pub fn get_config() -> u32 {
    let args = App::new("ssg-deployer(host)")
    .version(clap::crate_version!())
    .author(clap::crate_authors!())
    .about("simple deployer")
    .args(
        &[
        Arg::from_usage("[port] -p --port <integer> 'sets port to listen'")
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
        ]
    )
    .get_matches();

    return args.value_of("port").unwrap().parse::<u32>().unwrap();
}