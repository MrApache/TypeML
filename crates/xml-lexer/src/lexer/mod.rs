mod attribute;
mod error_impls;
mod tag;
mod tag_end;
mod token;
mod utils;
mod value;

use crate::layout_reader::XmlLexer;
use crate::position::*;
use std::io::Read;

pub use attribute::Attribute;
pub use tag::Tag;
pub use tag_end::TagEnd;
pub use token::Token;
pub use value::Value;

use crate::errors::XmlLayoutError;
use crate::lexer::utils::{is_name_char, is_name_start_char, is_valid_xml_text_char};

impl<'a> XmlLexer<'a> {
    pub fn location(&self) -> Location {
        Location {
            line_position: self.start_of_line.min(self.current_span.start),
            line: self.location.line,
            column: self.location.column.max(1),
        }
    }

    pub const fn file(&self) -> &str {
        self.file.as_str()
    }

    fn consume_byte(&mut self, skip_whitespaces: bool) -> Result<u8, XmlLayoutError> {
        let mut buffer = [0u8; 1];

        loop {
            let result = match self.inner.read(&mut buffer) {
                Ok(0) => return Err(self.err_unexpected_eof()),
                _ => Ok(buffer[0]),
            };

            let byte = buffer[0];

            if byte == b'\n' {
                self.start_of_line = self.inner.position() as usize - 1;
                self.location.line += 1;
                self.location.column = 0; // set to 0 because cursor stayed at \n
            } else {
                self.location.column += 1;
            }

            if !skip_whitespaces {
                return result;
            }

            if byte == b' ' || byte == b'\t' || byte == b'\r' || byte == b'\n' {
                continue;
            }

            return result;
        }
    }

    fn peek_byte_safe(&mut self) -> Option<u8> {
        let mut buffer = [0u8; 1];
        let current_pos = self.inner.position();
        let result = match self.inner.read(&mut buffer) {
            Ok(0) => None,
            _ => Some(buffer[0]),
        };
        self.inner.set_position(current_pos);
        result
    }

    fn peek_byte(&mut self) -> Result<u8, XmlLayoutError> {
        let mut buffer = [0u8; 1];
        let current_pos = self.inner.position();
        let result = match self.inner.read(&mut buffer) {
            Ok(0) => return Err(self.err_unexpected_eof()),
            _ => Ok(buffer[0]),
        };
        self.inner.set_position(current_pos);
        result
    }

    fn peek_byte_no_ws(&mut self) -> Result<u8, XmlLayoutError> {
        let mut buffer = [0u8; 1];
        let current_pos = self.inner.position();
        loop {

            if let Ok(0) = self.inner.read(&mut buffer) {
                return Err(self.err_unexpected_eof())
            }

            if buffer[0] == b' ' || buffer[0] == b'\t' || buffer[0] == b'\r' || buffer[0] == b'\n' {
                continue;
            }

            break;
        }

        self.inner.set_position(current_pos);
        Ok(buffer[0])
    }

    fn read_tag_span(&mut self) -> Result<bool, XmlLayoutError> {
        let mut is_open = true;
        self.consume_byte(true)?; // Skip '<' with whitespaces

        let old_location = self.location;
        let start = self.cursor_position();
        let mut end = start;

        if self.peek_byte_no_ws()? == b'/' {
            self.consume_byte(true)?; // Skip '/' with whitespaces
            is_open = false;
        }

        let mut inside_quotes = false;
        let mut slash = false;
        loop {
            self.current_span = Span::new(start as usize - 1, end as usize + 1);
            let byte = self.consume_byte(true)?;
            end = self.cursor_position();
            match byte {
                b'"' => inside_quotes = !inside_quotes,
                b'>' => {
                    if inside_quotes {
                        continue;
                    }
                    break;
                }
                b'/' => {
                    if inside_quotes {
                        continue;
                    }
                    if slash {
                        return Err(self.err_unexpected_char('>', '/'));
                    }
                    slash = true;
                }
                _ => {}
            }
        }
        self.inner.set_position(start);
        self.location = old_location;
        Ok(is_open)
    }

    fn text(&mut self) -> Result<Token, XmlLayoutError> {
        let start = self.cursor_position();
        loop {
            let byte = self.peek_byte_safe();
            if byte.is_none() {
                break;
            }
            match byte.unwrap() {
                b'<' | b'&' => {
                    break;
                }
                b => {
                    if is_valid_xml_text_char(b as char) {
                        self.consume_byte(false)?;
                    } else {
                        return Err(self.err_invalid_char(b as char));
                    }
                }
            }
        }

        self.current_span.start = start as usize + 1;
        self.current_span.end = self.cursor_position() as usize;
        Ok(Token::Text(self.substring()))
    }

    fn peek_comment(&mut self) -> Result<bool, XmlLayoutError> {
        let location = self.location;
        let position = self.cursor_position();
        if self.peek_byte_no_ws()? == b'<' {
            self.consume_byte(true)?;
        } else {
            return Ok(false);
        }

        if self.peek_byte()? == b'!' {
            self.consume_byte(false)?;
        } else {
            self.inner.set_position(position);
            self.location = location;
            return Ok(false);
        }

        for _ in 0..2 {
            if self.peek_byte()? == b'-' {
                self.consume_byte(false)?;
            } else {
                self.inner.set_position(position);
                self.location = location;
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn read_comment(&mut self) -> Result<Token, XmlLayoutError> {
        self.consume_byte(true)?; // Skip '<' with whitespaces
        let mut buffer = [0u8; 2];
        let mut first = true;
        loop {
            let byte = self.peek_byte().unwrap();
            if byte == b'>' && buffer[0] == b'-' && buffer[1] == b'-' {
                self.consume_byte(false)?;
                return Ok(Token::Comment);
            }
            if first {
                buffer[0] = byte;
                first = false;
            } else {
                buffer[1] = byte;
                first = true;
            }

            self.consume_byte(false)?;
        }
    }

    pub fn read(&mut self) -> Result<Token, XmlLayoutError> {
        match self.peek_byte_no_ws() {
            Ok(b'<') => {
                if self.peek_comment()? {
                    self.read_comment()
                } else if self.read_tag_span()? {
                    self.read_tag()
                } else {
                    self.read_tag_end()
                }
            }
            Err(e) => match e {
                XmlLayoutError::EndOfFile { .. } => Ok(Token::Eof),
                _ => Err(e),
            },
            Ok(other) => {
                let byte = self.peek_byte_no_ws().unwrap();
                match byte {
                    b'<' | b'&' => {
                        todo!("not impl: {}", other as char)
                    }
                    b => {
                        if is_valid_xml_text_char(b as char) {
                            self.text()
                        } else {
                            Err(self.err_invalid_char(b as char))
                        }
                    }
                }
            }
        }
    }

    fn cursor_position(&self) -> u64 {
        self.inner.position()
    }

    pub fn substring(&self) -> String {
        self.inner.get_ref()[self.current_span.start..self.current_span.end].to_string()
    }

    pub fn substring_other(&self, span: &Span) -> String {
        self.inner.get_ref()[span.start..span.end].to_string()
    }

    fn read_tag_end(&mut self) -> Result<Token, XmlLayoutError> {
        self.consume_byte(true)?; // Skip '/' with whitespaces

        let span = self.current_span;
        self.current_span = self.read_tag_identifier(false)?;
        Ok(Token::TagEnd(TagEnd {
            span,
            identifier: self.substring(),
        }))
    }

    fn read_tag(&mut self) -> Result<Token, XmlLayoutError> {
        self.skip_whitespaces()?;
        let mut location = self.location();
        location.column += 1;

        let identifier = self.read_tag_identifier(true)?;
        let attributes = self.read_attributes()?;

        let mut byte = self.peek_byte_no_ws()?;
        if byte == b'/' {
            self.consume_byte(true)?;
            byte = self.peek_byte_no_ws()?;
            if byte == b'>' {
                self.consume_byte(true)?;
                Ok(Token::TagEmpty(Tag {
                    span: self.current_span,
                    location,
                    identifier: self.substring_other(&identifier),
                    attributes,
                }))
            } else {
                Err(self.err_unexpected_char_with_loc(location, '>', byte as char))
            }
        } else if byte == b'>' {
            Ok(Token::TagStart(Tag {
                span: self.current_span,
                location,
                identifier: self.substring_other(&identifier),
                attributes,
            }))
        } else {
            Err(self.err_unexpected_char_with_loc(location, '>', byte as char))
        }
    }

    fn read_tag_identifier(&mut self, is_start: bool) -> Result<Span, XmlLayoutError> {
        self.read_identifier_first_char()?;
        let start = self.cursor_position() as usize - 1;
        let mut end = start + 1;

        loop {
            match self.consume_byte(false)? {
                b'_' => end += 1,
                b'-' => end += 1,
                b'.' => end += 1,
                b if b.is_ascii_alphabetic() => end += 1,
                b if b.is_ascii_digit() => end += 1,
                b if b.is_ascii_whitespace() => return Ok(Span { start, end }),
                b'>' => {
                    self.inner.set_position(end as u64);
                    return Ok(Span { start, end });
                }
                b'/' if is_start => {
                    self.inner.set_position(end as u64);
                    return Ok(Span { start, end });
                }

                other => return Err(self.err_expected_identifier(other as char)),
            }
        }
    }

    fn read_attributes(&mut self) -> Result<Vec<Attribute>, XmlLayoutError> {
        let mut attributes = Vec::new();
        while !self.peek_end_of_tag()? {
            self.skip_whitespaces()?;
            let mut attribute_location = self.location();
            attribute_location.column += 1;
            let attribute_name = self.read_attribute_identifier()?;
            self.consume_equal_char()?;
            let attribute_value = self.read_attribute_value()?;
            attributes.push(Attribute {
                span: self.current_span,
                location: attribute_location,
                name: attribute_name,
                value: attribute_value,
            })
        }
        Ok(attributes)
    }

    fn read_identifier_first_char(&mut self) -> Result<char, XmlLayoutError> {
        let first = self.consume_byte(true)?;
        if !is_name_start_char(first) {
            return Err(self.err_expected_identifier(first as char));
        }

        Ok(first as char)
    }

    fn read_attribute_identifier(&mut self) -> Result<Value, XmlLayoutError> {
        let mut identifier = String::new();
        let first = self.read_identifier_first_char()?;
        identifier.push(first);

        let location = self.location();
        let start = location.column + location.position();

        loop {
            let byte = self.peek_byte()?;
            if is_name_char(byte) {
                self.consume_byte(false)?;
                identifier.push(byte as char);
            } else {
                break;
            }
        }

        Ok(Value {
            location,
            span: Span::new(start, start + identifier.len()),
            inner: identifier,
        })
    }

    fn error_span(&self, length: usize) -> ErrorSpan {
        let inner = self.substring();
        let start = self.location.column
            - (self.current_span.start - self.start_of_line.min(self.current_span.start));
        let source = inner.trim().to_string();
        ErrorSpan {
            source,
            start,
            length,
        }
    }

    fn read_attribute_value(&mut self) -> Result<Value, XmlLayoutError> {
        let quote = self.consume_byte(true)?;
        if quote != b'"' && quote != b'\'' {
            return Err(self.err_unexpected_char('"', quote as char));
        }

        let mut location = self.location();
        location.column += 1;
        let start = location.column + location.position();

        let mut value = String::new();

        loop {
            let byte = self.consume_byte(false)?;
            if byte == quote {
                break;
            }

            value.push(byte as char);
        }

        Ok(Value {
            location,
            span: Span::new(start, start + value.len()),
            inner: value,
        })
    }

    fn peek_end_of_tag(&mut self) -> Result<bool, XmlLayoutError> {
        let byte = self.peek_byte_no_ws()?;
        Ok(byte == b'/' || byte == b'>')
    }

    fn consume_equal_char(&mut self) -> Result<(), XmlLayoutError> {
        let byte = self.peek_byte()?;
        self.consume_byte(false)?;
        if byte == b'=' {
            Ok(())
        } else {
            Err(self.err_unexpected_char('=', byte as char))
        }
    }

    fn skip_whitespaces(&mut self) -> Result<(), XmlLayoutError> {
        while self.peek_byte()?.is_ascii_whitespace() {
            self.consume_byte(false)?;
        }
        Ok(())
    }
}
