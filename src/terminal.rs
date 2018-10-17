use console::Term;
use std::cmp::{max, min};
use std::io::{self, Write};
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

    pub fn should_retry(&mut self, running: bool) -> Result<bool, io::Error> {
        if self.width.is_none() {
            return Ok(false);
        }

        println!();
        loop {
            let _ = self
                .stdout
                .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true));
            let _ = write!(&mut self.stdout, "Retry? [y/N] ");
            if !running {
                write!(&mut self.stdout, "(Press Ctrl+C again to stop Topgrade) ");
            }
            let _ = self.stdout.reset();
            let _ = self.stdout.flush();

            match Term::stdout().read_char()? {
                'y' | 'Y' => return Ok(true),
                'n' | 'N' | '\r' | '\n' => return Ok(false),
                _ => (),
            }
        }
    }
}
