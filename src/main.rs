mod mbox;
mod myimap;

use config;
use imap::types::Flag;
use mbox::Mbox;
use myimap::{imap_session, ImapConfig};

fn get_imap_config() -> ImapConfig {
    let mut builder = config::Config::default();
    let settings = builder
        .merge(config::File::with_name("./Settings.toml"))
        .unwrap();
    ImapConfig {
        domain: settings.get_str("domain").expect("Has setting"),
        port: settings.get_int("port").expect("Has setting") as u16,
        username: settings.get_str("username").expect("Has setting"),
        password: settings.get_str("password").expect("Has setting"),
    }
}

fn main() -> std::io::Result<()> {
    let mbox = Mbox::new("all.mbox");
    let imap_config = get_imap_config();
    let mut session = imap_session(imap_config);
    for mail in mbox.into_iter().take(3) {
        println!("{:?} len: {}", mail.from_line().unwrap(), mail.lines.len());
        match session.append_with_flags("inbox", mail.as_body(), &[Flag::Seen, Flag::Answered]) {
            Err(error) => panic!("{:?}", error),
            _ => println!("Message appended!"),
        }
    }
    Ok(())
}
