use chrono::{Local, Timelike};
use console::{style, Term};
use lazy_static::lazy_static;
use std::cmp::{max, min};
use std::env;
use std::io::{self, Write};
use std::process::Command;
use std::sync::Mutex;
#[cfg(windows)]
use which_crate::which;

lazy_static! {
    static ref TERMINAL: Mutex<Terminal> = Mutex::new(Terminal::new());
}

#[cfg(unix)]
pub fn shell() -> String {
    env::var("SHELL").unwrap_or_else(|_| "sh".to_string())
}

#[cfg(windows)]
pub fn shell() -> &'static str {
    which("pwsh").map(|_| "pwsh").unwrap_or("powershell")
}

pub fn run_shell() {
    Command::new(shell()).spawn().unwrap().wait().unwrap();
}

struct Terminal {
    width: Option<u16>,
    prefix: String,
    term: Term,
    set_title: bool,
}

impl Terminal {
    fn new() -> Self {
        let term = Term::stdout();
        Self {
            width: term.size_checked().map(|(_, w)| w),
            term,
            prefix: env::var("TOPGRADE_PREFIX")
                .map(|prefix| format!("({}) ", prefix))
                .unwrap_or_else(|_| String::new()),
            set_title: true,
        }
    }

    fn set_title(&mut self, set_title: bool) {
        self.set_title = set_title
    }

    fn print_separator<P: AsRef<str>>(&mut self, message: P) {
        if self.set_title {
            self.term
                .set_title(format!("{}Topgrade - {}", self.prefix, message.as_ref()));
        }
        let now = Local::now();
        let message = format!(
            "{}{:02}:{:02}:{:02} - {}",
            self.prefix,
            now.hour(),
            now.minute(),
            now.second(),
            message.as_ref()
        );
        match self.width {
            Some(width) => {
                self.term
                    .write_fmt(format_args!(
                        "{}\n",
                        style(format_args!(
                            "\n―― {} {:―^border$}",
                            message,
                            "",
                            border = max(
                                2,
                                min(80, width as usize)
                                    .checked_sub(4)
                                    .and_then(|e| e.checked_sub(message.len()))
                                    .unwrap_or(0)
                            )
                        ))
                        .bold()
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

    #[allow(dead_code)]
    fn print_info<P: AsRef<str>>(&mut self, message: P) {
        let message = message.as_ref();
        self.term
            .write_fmt(format_args!("{}\n", style(message).blue().bold()))
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

    fn should_retry(&mut self, interrupted: bool) -> Result<bool, io::Error> {
        if self.width.is_none() {
            return Ok(false);
        }

        if self.set_title {
            self.term.set_title("Topgrade - Awaiting user");
        }
        self.term
            .write_fmt(format_args!(
                "\n{}",
                style(format!(
                    "{}Retry? (y)es/(N)o/(s)hell {}",
                    self.prefix,
                    if interrupted {
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
                's' | 'S' => {
                    println!("\n\nDropping you to shell. Fix what you need and then exit the shell.\n");
                    run_shell();
                    break Ok(true);
                }
                'n' | 'N' | '\r' | '\n' => break Ok(false),
                _ => (),
            }
        };

        self.term.write_str("\n").ok();

        answer
    }

    fn get_char(&self) -> Result<char, io::Error> {
        self.term.read_char()
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}

pub fn should_retry(interrupted: bool) -> Result<bool, io::Error> {
    TERMINAL.lock().unwrap().should_retry(interrupted)
}

pub fn print_separator<P: AsRef<str>>(message: P) {
    TERMINAL.lock().unwrap().print_separator(message)
}

#[allow(dead_code)]
pub fn print_warning<P: AsRef<str>>(message: P) {
    TERMINAL.lock().unwrap().print_warning(message)
}

#[allow(dead_code)]
pub fn print_info<P: AsRef<str>>(message: P) {
    TERMINAL.lock().unwrap().print_info(message)
}

pub fn print_result<P: AsRef<str>>(key: P, succeeded: bool) {
    TERMINAL.lock().unwrap().print_result(key, succeeded)
}

/// Tells whether the terminal is dumb.
pub fn is_dumb() -> bool {
    TERMINAL.lock().unwrap().width.is_none()
}

pub fn get_char() -> char {
    TERMINAL.lock().unwrap().get_char().unwrap()
}

pub fn set_title(set_title: bool) {
    TERMINAL.lock().unwrap().set_title(set_title);
}
