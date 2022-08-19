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

use chrono::{FixedOffset, TimeZone};
use config::{self, Config, ConfigError, File, FileFormat};
use imap::types::Flag;
use imap::Error::No;
use mbox::{Mbox, MboxConfig};
use myimap::{imap_session, ImapConfig};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Settings {
    mbox: MboxConfig,
    imap: ImapConfig,
}

impl Settings {
    fn load() -> Result<Self, ConfigError> {
        let builder = Config::builder().add_source(File::new("./Settings.toml" , FileFormat::Toml));
        builder.build()?.try_deserialize()
    }
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = Settings::load()?;

    let mbox = Mbox::new(&config.mbox.path);

    let mut session = imap_session(&config.imap);

    let tz_offset = FixedOffset::east(config.imap.tz_offset as i32);

    let mbox_sent = "\\Sent".to_string();
    let my_emails: Vec<String> = config.mbox.emails;

    for mail in mbox.into_iter() {
        let append_to_box = if my_emails.contains(&mail.from) {
            &mbox_sent
        } else {
            &config.mbox.dest
        };
        println!("{} len: {} → {}", mail, mail.lines.len(), append_to_box);
        match session.append_with_flags_and_date(
            append_to_box,
            mail.as_body(),
            &[Flag::Seen, Flag::Answered],
            Some(tz_offset.from_local_datetime(&mail.date).unwrap()),
        ) {
            Err(error) => match error {
                No(ref msg) => println!("Skipping: {:?}", msg),
                _ => panic!("{:?}", error),
            },
            _ => (),
        }
    }
    Ok(())
}
