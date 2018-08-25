use std::cmp::{max, min};
use std::io::{stdin, Write};
use term_size;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct Terminal {
    width: Option<usize>,
    stdout: StandardStream,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            width: term_size::dimensions().map(|(w, _)| w),
            stdout: StandardStream::stdout(ColorChoice::Auto),
        }
    }

    pub fn print_separator<P: AsRef<str>>(&mut self, message: P) {
        let message = message.as_ref();
        match self.width {
            Some(width) => {
                let _ = self
                    .stdout
                    .set_color(ColorSpec::new().set_fg(Some(Color::White)).set_bold(true));
                let _ = writeln!(
                    &mut self.stdout,
                    "\n―― {} {:―^border$}",
                    message,
                    "",
                    border = max(2, min(80, width as usize) - 3 - message.len())
                );
                let _ = self.stdout.reset();
                let _ = self.stdout.flush();
            }
            None => {
                let _ = writeln!(&mut self.stdout, "―― {} ――", message);
            }
        }
    }

    #[allow(dead_code)]
    pub fn print_warning<P: AsRef<str>>(&mut self, message: P) {
        let message = message.as_ref();

        let _ = self
            .stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true));
        let _ = writeln!(&mut self.stdout, "{}", message);
        let _ = self.stdout.reset();
        let _ = self.stdout.flush();
    }

    pub fn print_result<P: AsRef<str>>(&mut self, key: P, succeeded: bool) {
        let key = key.as_ref();
        let _ = write!(&mut self.stdout, "{}: ", key);

        let _ = self.stdout.set_color(
            ColorSpec::new()
                .set_fg(Some(if succeeded { Color::Green } else { Color::Red }))
                .set_bold(true),
        );

        let _ = writeln!(&mut self.stdout, "{}", if succeeded { "OK" } else { "FAILED" });

        let _ = self.stdout.reset();
        let _ = self.stdout.flush();
    }

    pub fn should_retry(&mut self) -> bool {
        if self.width.is_none() {
            return false;
        }
        println!("");
        loop {
            let mut result = String::new();

            let _ = self
                .stdout
                .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true));
            let _ = write!(&mut self.stdout, "Retry? [y/N] ");
            let _ = self.stdout.reset();
            let _ = self.stdout.flush();

            if let Ok(_) = stdin().read_line(&mut result) {
                match result.as_str() {
                    "y\n" => return true,
                    "n\n" | "\n" => return false,
                    _ => (),
                }
            } else {
                return false;
            }
        }
    }
}
