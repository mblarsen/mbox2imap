use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Mail {
    pub lines: Vec<String>,
}

impl Mail {
    pub fn from_line(&self) -> Option<&String> {
        self.lines.get(0)
    }

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
            from_line_regex: Regex::new(r"^From ").unwrap(),
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
                let mail = Mail {
                    lines: mail_lines.clone(),
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
            let mail = Mail {
                lines: mail_lines.clone(),
            };
            self.left = vec![];
            Some(mail)
        } else {
            None
        }
    }
}
