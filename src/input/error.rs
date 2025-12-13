use std::{
    fmt::Display,
    io::{self, ErrorKind},
};

use serde_json::error::Category;

pub type InputResult<T> = Result<T, InputError>;

#[derive(Debug)]
pub enum InputError {
    FileNotFound,
    PermissionError,
    UnknownError,

    JSONSyntaxError,
    JSONSemanticsError,
    JSONIOError,
}

impl Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use InputError::*;
        write!(
            f,
            "{}",
            match self {
                FileNotFound => "the settings file was not found in the current directory",
                PermissionError =>
                    "the process didnt had the permission to access the settings file",
                UnknownError => "an unknown error occured in the process",

                JSONSyntaxError => "the settings file didn't contain valid JSON syntax",
                JSONSemanticsError => "the settings file didn't contain semantically correct JSON",
                JSONIOError => "an IO error occured in the process of processing the JSON",
            }
        )
    }
}

impl From<serde_json::Error> for InputError {
    fn from(value: serde_json::Error) -> Self {
        match value.classify() {
            Category::Syntax => InputError::JSONSyntaxError,
            Category::Data => InputError::JSONSemanticsError,
            Category::Io => InputError::JSONIOError,
            Category::Eof => InputError::JSONSyntaxError,
        }
    }
}

impl From<io::Error> for InputError {
    fn from(value: io::Error) -> Self {
        match value.kind() {
            ErrorKind::NotFound => InputError::FileNotFound,
            ErrorKind::PermissionDenied => InputError::PermissionError,
            _ => InputError::UnknownError,
        }
    }
}
