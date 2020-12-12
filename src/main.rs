mod mbox;
mod myimap;

use config::{self, Config, ConfigError};
use imap::types::Flag;
use mbox::Mbox;
use myimap::{imap_session, ImapConfig};

fn load_config() -> Result<Config, ConfigError> {
    let mut builder = config::Config::default();
    Ok(builder
        .merge(config::File::with_name("./Settings.toml"))
        .unwrap()
        .clone())
}

fn main() -> std::io::Result<()> {
    let config = match load_config() {
        Ok(config) => config,
        Err(error) => panic!("{:?}", error),
    };

    let mbox = Mbox::new(
        config
            .get_str("mbox_path")
            .expect("Mbox path is defined")
            .as_str(),
    );

    let mut session = imap_session(ImapConfig {
        domain: config.get_str("domain").expect("IMAP domain is provided"),
        port: config.get_int("port").expect("IMAP port is provided") as u16,
        username: config
            .get_str("username")
            .expect("IMAP username is provided"),
        password: config
            .get_str("password")
            .expect("IMAP username is provided"),
    });

    for mail in mbox.into_iter().take(3) {
        println!("{} len: {}", mail, mail.lines.len());
        match session.append_with_flags("inbox", mail.as_body(), &[Flag::Seen, Flag::Answered]) {
            Err(error) => panic!("{:?}", error),
            _ => println!("Message appended!"),
        }
    }
    Ok(())
}
