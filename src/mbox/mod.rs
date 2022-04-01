use chrono::NaiveDateTime;
use regex::Regex;
use serde::Deserialize;
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize)]
pub struct MboxConfig {
    pub path: String,
    pub dest: String,
    pub emails: Vec<String>,
    pub dryrun: bool,
    pub date_format: String,
    pub before_date: Option<String>,
    pub after_date: Option<String>,
}

#[derive(Debug)]
pub struct Mail {
    pub from: String,
    pub date: NaiveDateTime,
    pub lines: Vec<String>,
}

impl Display for Mail {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "From: {}, sent: {}", self.from, self.date)
    }
}

impl Mail {
    pub fn as_body(&self) -> String {
        self.lines.join("\r\n")
    }
}

pub struct Mbox {
    pub from_line_regex: Regex,
    pub path: String,
    pub buffer: BufReader<File>,
    pub left: Vec<String>,
    pub lines_read: usize,
    pub date_format: String,
}

impl Mbox {
    pub fn new(path: &str, date_format: &str) -> Self {
        Self {
            // Gmail format
            from_line_regex: Regex::new(r"^From ([^\s]+) (.{30})").unwrap(),
            // Protonmail format
            // from_line_regex: Regex::new(r"^From ([^\s]+) (.{24})").unwrap(),
            path: path.to_owned(),
            buffer: BufReader::new(File::open(path).expect("File exists")),
            left: Vec::new(),
            lines_read: 0,
            date_format: date_format.to_owned(),
        }
    }

    fn is_from_line(&self, line: &str) -> bool {
        self.from_line_regex.is_match(line)
    }
}

impl std::iter::Iterator for Mbox {
    type Item = Mail;

    fn next(&mut self) -> Option<Self::Item> {
        let mut mail_lines = self.left.clone();
        let mut line = String::new();

        while self.buffer.read_line(&mut line).expect("File is readable") != 0 {
            self.lines_read += 1;
            if self.lines_read != 1 && self.is_from_line(&line) {
                self.left = vec![line.trim().to_owned()];
                let captures = self
                    .from_line_regex
                    .captures(mail_lines[0].as_str())
                    .expect("Must have from email and date");

                let mail = Mail {
                    lines: mail_lines.clone(),
                    from: captures
                        .get(1)
                        .map(|m| String::from(m.as_str()))
                        .expect("Must have from email"),
                    date: captures
                        .get(2)
                        .map(|m| {
                            NaiveDateTime::parse_from_str(m.as_str(), &self.date_format)
                                .expect("Date can be parsed")
                        })
                        .expect("Must have date"),
                };
                mail_lines.clear();
                return Some(mail);
            } else {
                mail_lines.push(line.trim().to_owned());
                line = String::new();
            }
        }

        // TODO figure out away to avoid this last check
        if mail_lines.len() > 0 {
            let captures = self
                .from_line_regex
                .captures(mail_lines[0].as_str())
                .expect("Must have from email and date");
            print!("Date {:?}", mail_lines[0].as_str());
            let mail = Mail {
                lines: mail_lines.clone(),
                from: captures
                    .get(1)
                    .map(|m| String::from(m.as_str()))
                    .expect("Must have from email"),
                date: captures
                    .get(2)
                    .map(|m| {
                        NaiveDateTime::parse_from_str(m.as_str(), &self.date_format)
                            .expect("Date can be parsed")
                    })
                    .expect("Must have date"),
            };
            self.left = vec![];
            Some(mail)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveDateTime};

    #[test]
    fn can_parse_date() {
        assert_eq!(
            NaiveDateTime::parse_from_str("Thu Mar  8 04:14:36 2018", "%a %b %e %T %Y"),
            Ok(NaiveDate::from_ymd(2018, 3, 8).and_hms(4, 14, 36))
        );
    }
}
