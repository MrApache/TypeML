mod error_context;
mod io_error;

pub use error_context::ErrorContext;
use io_error::IoError;

use crate::position::Location;

#[non_exhaustive]
#[derive(PartialEq, Debug)]
pub enum XmlLayoutError {
    Io(IoError),
    Utf8Error(std::str::Utf8Error),

    UnexpectedChar {
        context: ErrorContext,
        expected: char,
        found: char,
    },
    ExpectedIdentifier {
        context: ErrorContext,
        found: char,
    },
    InvalidChar {
        context: ErrorContext,
        char: char,
    },

    EndOfFile {
        file: String,
        location: Location,
    },
}

impl From<std::io::Error> for XmlLayoutError {
    fn from(error: std::io::Error) -> Self {
        XmlLayoutError::Io(IoError(error))
    }
}

impl std::error::Error for XmlLayoutError {}

impl std::fmt::Display for XmlLayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XmlLayoutError::Io(error) => write!(f, "Could not load file: {error}"),
            XmlLayoutError::Utf8Error(error) => write!(f, "Could not read file: {error}"),
            XmlLayoutError::UnexpectedChar {
                context,
                expected,
                found,
            } => write_single_error(
                format!("Unexpected char: Expected '{expected}', but found '{found}'"),
                context,
                f,
            ),

            XmlLayoutError::ExpectedIdentifier { context, found } => write_single_error(
                format!("Expected identifier, but found '{found}'"),
                context,
                f,
            ),

            XmlLayoutError::InvalidChar { context, char } => {
                write_single_error(format!("Invalid character: '{char}'"), context, f)
            }

            XmlLayoutError::EndOfFile { file, location } => {
                write!(f, "[{file}:{location}] Unexpected end of file")
            }
        }
    }
}

fn write_single_error(
    message: impl Into<String>,
    context: &ErrorContext,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    let file = &context.file;
    let location = &context.location;
    let error = &context.error;

    writeln!(f, "error: {}", message.into())?;
    writeln!(f, "  --> {file}:{}:{}", location.line(), location.column())?;
    writeln!(f, "   |")?;
    writeln!(f, "   |      {}", error.source())?;
    writeln!(
        f,
        "   |      {}{}",
        " ".repeat(error.start()),
        "^".repeat(error.length())
    )?;
    Ok(())
}
