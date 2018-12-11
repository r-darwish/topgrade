use console::{style, Term};
use lazy_static::lazy_static;
use std::cmp::{max, min};
use std::io::{self, Write};
use std::sync::Mutex;

lazy_static! {
    static ref TERMINAL: Mutex<Terminal> = Mutex::new(Terminal::new());
}

struct Terminal {
    width: Option<u16>,
    term: Term,
}

impl Terminal {
    fn new() -> Self {
        let term = Term::stdout();
        Self {
            width: term.size_checked().map(|(_, w)| w),
            term,
        }
    }

    fn print_separator<P: AsRef<str>>(&mut self, message: P) {
        let message = message.as_ref();
        match self.width {
            Some(width) => {
                self.term
                    .write_fmt(format_args!(
                        "{}\n",
                        style(format_args!(
                            "\n―― {} {:―^border$}",
                            message,
                            "",
                            border = max(2, min(80, width as usize) - 3 - message.len())
                        ))
                        .bold()
                        .white()
                    ))
                    .ok();
            }
            None => {
                self.term.write_fmt(format_args!("―― {} ――\n", message)).ok();
            }
        }
    }

    #[allow(dead_code)]
    fn print_warning<P: AsRef<str>>(&mut self, message: P) {
        let message = message.as_ref();
        self.term
            .write_fmt(format_args!("{}\n", style(message).yellow().bold()))
            .ok();
    }

    fn print_result<P: AsRef<str>>(&mut self, key: P, succeeded: bool) {
        let key = key.as_ref();

        self.term
            .write_fmt(format_args!(
                "{}: {}\n",
                key,
                if succeeded {
                    style("OK").bold().green()
                } else {
                    style("FAILED").bold().red()
                }
            ))
            .ok();
    }

    fn should_retry(&mut self, running: bool) -> Result<bool, io::Error> {
        if self.width.is_none() {
            return Ok(false);
        }

        self.term
            .write_fmt(format_args!(
                "\n{}",
                style(format!(
                    "Retry? [y/N] {}",
                    if !running {
                        "(Press Ctrl+C again to stop Topgrade) "
                    } else {
                        ""
                    }
                ))
                .yellow()
                .bold()
            ))
            .ok();

        let answer = loop {
            match self.term.read_char()? {
                'y' | 'Y' => break Ok(true),
                'n' | 'N' | '\r' | '\n' => break Ok(false),
                _ => (),
            }
        };

        self.term.write_str("\n").ok();

        answer
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}

pub fn should_retry(running: bool) -> Result<bool, io::Error> {
    TERMINAL.lock().unwrap().should_retry(running)
}

pub fn print_separator<P: AsRef<str>>(message: P) {
    TERMINAL.lock().unwrap().print_separator(message)
}

pub fn print_warning<P: AsRef<str>>(message: P) {
    TERMINAL.lock().unwrap().print_warning(message)
}

pub fn print_result<P: AsRef<str>>(key: P, succeeded: bool) {
    TERMINAL.lock().unwrap().print_result(key, succeeded)
}
