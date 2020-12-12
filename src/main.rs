//! A simple mbox to IMAP migration tool
//!
//! Lets you:
//! - specify target mail box, defaults to \Archive
//! - direct your own email(s) to \Sent
//! - sets original send/receive date
//!
//! ## Usage
//!
//! ```shell
//! cp Settings.example.toml Settings.toml
//! ```
//!
//! Update settings to your setup.
//!
//! Then run it!
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

    let mbox_sent = "\\Sent".to_string();
    let mbox_dest = config
        .get_str("mbox_dest")
        .expect("Mbox destination is provided");
    let my_emails: Vec<String> = config
        .get_array("my_emails")
        .expect("My emails is provided")
        .into_iter()
        .map(|val| val.into_str().expect("Is email"))
        .collect();
    for mail in mbox.into_iter() {
        let append_to_box = if my_emails.contains(&mail.from) {
            &mbox_sent
        } else {
            &mbox_dest
        };
        println!("{} len: {} → {}", mail, mail.lines.len(), append_to_box);
        match session.append_with_flags_and_date(
            append_to_box,
            mail.as_body(),
            &[Flag::Seen, Flag::Answered],
            Some(mail.date),
        ) {
            Err(error) => panic!("{:?}", error),
            _ => (),
        }
    }
    Ok(())
}
