/*

#[derive(Debug)]
pub struct RmlError {
    file: String,
    kind: RmlParserError,
}


#[derive(PartialEq, Debug)]
pub enum RmlParserError {
    LexerError(RmlLexerError),
    UnknownAttributes(Vec<Attribute>),
}

//impl From<std::io::Error> for RmlParserError {
//    fn from(error: std::io::Error) -> Self {
//        RmlLexerError::Io(IoError(error))
//    }
//}

//impl std::error::Error for RmlParserError {}

impl From<RmlLexerError> for RmlParserError {
    fn from(value: RmlLexerError) -> Self {
        Self::LexerError(value)
    }
}

impl std::fmt::Display for RmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            RmlParserError::LexerError(rml_lexer_error) => {
                match rml_lexer_error {
                    RmlLexerError::Io(error) => write!(f, "Could not load file: {error}"),
                    RmlLexerError::Utf8Error(error) => write!(f, "Could not read file: {error}"),
                    RmlLexerError::UnexpectedChar {
                        context,
                        expected,
                        found,
                    } => write_single_error(
                        &self.file,
                        format!("Unexpected char: Expected '{expected}', but found '{found}'"),
                        &context,
                        f,
                    ),

                    RmlLexerError::ExpectedIdentifier { context, found } => write_single_error(
                        &self.file,
                        format!("Expected identifier, but found '{found}'"),
                        &context,
                        f,
                    ),

                    RmlLexerError::InvalidChar { context, char } => {
                        write_single_error(&self.file, format!("Invalid character: '{char}'"), &context, f)
                    }

                    RmlLexerError::EndOfFile { location } => {
                        let file = &self.file;
                        write!(f, "[{file}:{location}] Unexpected end of file")
                    }
                }
            },
            RmlParserError::UnknownAttributes(attributes) => todo!(),
        }
    }
}

fn write_single_error(
    file: &str,
    message: impl Into<String>,
    context: &ErrorContext,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
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


pub trait DocumentResult<T> {
    fn with_file(self, file: impl Into<String>) -> Result<T, RmlError>;
}

impl<T> DocumentResult<T> for Result<T, RmlParserError> {
    fn with_file(self, file: impl Into<String>) -> Result<T, RmlError> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(RmlError {
                file: file.into(),
                kind: error,
            })
        }
    }
}

impl<T> DocumentResult<T> for Result<T, RmlLexerError> {
    fn with_file(self, file: impl Into<String>) -> Result<T, RmlError> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(RmlError {
                file: file.into(),
                kind: RmlParserError::LexerError(error),
            })
        }
    }
}

*/
