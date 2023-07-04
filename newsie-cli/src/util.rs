//! Utilities

use std::{fmt::Display, process::exit};

use colored::Colorize;

/// Prints an info message
pub fn info(msg: &str) {
    eprintln!("{} {msg}", "i".yellow());
}

/// Prints a success message
pub fn success(msg: &str) {
    eprintln!("{} {msg}", "✔️".green());
}

/// Prints a warning message
pub fn warn(msg: &str) {
    eprintln!("{} {}", "!".yellow(), msg.yellow());
}

/// Prints an error message
pub fn error(msg: &str) {
    eprintln!("{} {}", "x".red(), msg.red());
}

/// Result extension trait
pub trait ResultExt<T, E>
where
    Self: Sized,
{
    /// Unwraps a result or exits with an error
    fn unwrap_or_exit(self) -> T;
}

impl<T, E> ResultExt<T, E> for Result<T, E>
where
    E: Display,
{
    fn unwrap_or_exit(self) -> T {
        match self {
            Ok(value) => value,
            Err(err) => {
                error(err.to_string().as_str());
                exit(1);
            }
        }
    }
}

/// Option extension trait
pub trait OptionExt<T> {
    /// Unwraps an option or exits with an error
    fn unwrap_or_exit(self, message: &str) -> T;
}

impl<T> OptionExt<T> for Option<T> {
    fn unwrap_or_exit(self, message: &str) -> T {
        match self {
            Some(value) => value,
            None => {
                error(message);
                exit(1);
            }
        }
    }
}
