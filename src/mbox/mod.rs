use chrono::NaiveDateTime;
use regex::Regex;
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};

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

#[derive(Debug)]
pub struct Mbox {
    from_line_regex: Regex,
    path: String,
    buffer: BufReader<File>,
    left: Vec<String>,
    lines_read: usize,
}

impl Mbox {
    pub fn new(path: &str) -> Self {
        Self {
            from_line_regex: Regex::new(r"^From ([^\s]+) (.{24})").unwrap(),
            path: path.to_owned(),
            buffer: BufReader::new(File::open(path).expect("File exists")),
            left: Vec::new(),
            lines_read: 0,
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
                            NaiveDateTime::parse_from_str(m.as_str(), "%a %b %e %T %Y")
                                .expect("Date is pareable")
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
            let mail = Mail {
                lines: mail_lines.clone(),
                from: captures
                    .get(1)
                    .map(|m| String::from(m.as_str()))
                    .expect("Must have from email"),
                date: captures
                    .get(2)
                    .map(|m| {
                        NaiveDateTime::parse_from_str(m.as_str(), "%a %b %e %T %Y")
                            .expect("Date is pareable")
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
