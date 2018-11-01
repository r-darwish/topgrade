use console::{style, Term};
use std::cmp::{max, min};
use std::io::{self, Write};

pub struct Terminal {
    width: Option<u16>,
    term: Term,
}

impl Terminal {
    pub fn new() -> Self {
        let term = Term::stdout();
        Self {
            width: term.size_checked().map(|(_, w)| w),
            term,
        }
    }

    pub fn print_separator<P: AsRef<str>>(&mut self, message: P) {
        let message = message.as_ref();
        match self.width {
            Some(width) => {
                println!(
                    "{}",
                    style(format!(
                        "\n―― {} {:―^border$}",
                        message,
                        "",
                        border = max(2, min(80, width as usize) - 3 - message.len())
                    )).bold()
                    .white()
                );
            }
            None => {
                println!("―― {} ――", message);
            }
        }
    }

    #[allow(dead_code)]
    pub fn print_warning<P: AsRef<str>>(&mut self, message: P) {
        let message = message.as_ref();
        println!("{}", style(message).yellow().bold());
    }

    pub fn print_result<P: AsRef<str>>(&mut self, key: P, succeeded: bool) {
        let key = key.as_ref();

        println!(
            "{}: {}",
            key,
            if succeeded {
                style("OK").bold().green()
            } else {
                style("FAILED").bold().red()
            }
        );
    }

    pub fn should_retry(&mut self, running: bool) -> Result<bool, io::Error> {
        if self.width.is_none() {
            return Ok(false);
        }

        println!();
        loop {
            self.term
                .write_fmt(format_args!(
                    "{}",
                    style(format!(
                        "Retry? [y/N] {}",
                        if !running {
                            "(Press Ctrl+C again to stop Topgrade) "
                        } else {
                            ""
                        }
                    )).yellow()
                    .bold()
                )).ok();

            match self.term.read_char()? {
                'y' | 'Y' => return Ok(true),
                'n' | 'N' | '\r' | '\n' => return Ok(false),
                _ => (),
            }
        }
    }
}
