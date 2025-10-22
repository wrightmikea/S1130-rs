//! Lexer for IBM 1130 Assembly Language
//!
//! Tokenizes assembly source code into a stream of tokens.

use crate::error::AssemblerError;

/// Result type for assembler operations
pub type Result<T> = std::result::Result<T, AssemblerError>;

/// Token types in IBM 1130 assembly
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// Label (identifier at start of line)
    Label(String),

    /// Instruction mnemonic (LD, STO, A, etc.)
    Instruction(String),

    /// Pseudo-op (ORG, DC, BSS, END, EQU)
    PseudoOp(String),

    /// Numeric literal (decimal, hex, or octal)
    Number(u16),

    /// Symbol/identifier
    Identifier(String),

    /// Comma separator
    Comma,

    /// Slash (indirect addressing)
    Slash,

    /// Asterisk (indirect addressing or comment)
    Asterisk,

    /// Newline
    Newline,

    /// End of file
    Eof,
}

/// Lexer state
pub struct Lexer {
    /// Source code
    source: Vec<char>,

    /// Current position
    position: usize,

    /// Current line number
    line: usize,

    /// Column in line
    column: usize,
}

impl Lexer {
    /// Create a new lexer
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            position: 0,
            line: 1,
            column: 0,
        }
    }

    /// Get current character without consuming
    fn peek(&self) -> Option<char> {
        self.source.get(self.position).copied()
    }

    /// Get next character without consuming
    fn peek_next(&self) -> Option<char> {
        self.source.get(self.position + 1).copied()
    }

    /// Consume and return current character
    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.position += 1;
        self.column += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 0;
        }
        Some(ch)
    }

    /// Skip whitespace (except newlines)
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skip to end of line (for comments)
    fn skip_to_eol(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    /// Read an identifier or keyword
    fn read_identifier(&mut self) -> String {
        let mut result = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        result
    }

    /// Read a number (decimal, hex, or octal)
    fn read_number(&mut self) -> Result<u16> {
        let start_line = self.line;

        // Check for hex (0x prefix)
        if self.peek() == Some('0') && matches!(self.peek_next(), Some('x') | Some('X')) {
            self.advance(); // consume '0'
            self.advance(); // consume 'x'

            let mut hex_str = String::new();
            while let Some(ch) = self.peek() {
                if ch.is_ascii_hexdigit() {
                    hex_str.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }

            return u16::from_str_radix(&hex_str, 16).map_err(|_| AssemblerError::SyntaxError {
                line: start_line,
                message: format!("Invalid hexadecimal number: 0x{}", hex_str),
            });
        }

        // Read decimal or octal
        let mut num_str = String::new();
        let mut is_octal = false;

        if self.peek() == Some('0') {
            is_octal = true;
        }

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if is_octal && num_str.len() > 1 {
            // Octal number (leading zero)
            u16::from_str_radix(&num_str[1..], 8).map_err(|_| AssemblerError::SyntaxError {
                line: start_line,
                message: format!("Invalid octal number: {}", num_str),
            })
        } else {
            // Decimal number
            num_str
                .parse::<u16>()
                .map_err(|_| AssemblerError::SyntaxError {
                    line: start_line,
                    message: format!("Invalid decimal number: {}", num_str),
                })
        }
    }

    /// Get next token
    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        match self.peek() {
            None => Ok(Token::Eof),

            Some('\n') => {
                self.advance();
                Ok(Token::Newline)
            }

            Some('*') if self.column == 0 => {
                // Comment at start of line
                self.skip_to_eol();
                // Consume the newline if present
                if self.peek() == Some('\n') {
                    self.advance();
                }
                Ok(Token::Newline)
            }

            Some('*') => {
                self.advance();
                Ok(Token::Asterisk)
            }

            Some('/') => {
                self.advance();
                Ok(Token::Slash)
            }

            Some(',') => {
                self.advance();
                Ok(Token::Comma)
            }

            Some(ch) if ch.is_ascii_digit() => {
                let num = self.read_number()?;
                Ok(Token::Number(num))
            }

            Some(ch) if ch.is_alphabetic() => {
                let ident = self.read_identifier();

                // Check if it's a pseudo-op
                if ident.eq_ignore_ascii_case("ORG")
                    || ident.eq_ignore_ascii_case("DC")
                    || ident.eq_ignore_ascii_case("BSS")
                    || ident.eq_ignore_ascii_case("END")
                    || ident.eq_ignore_ascii_case("EQU")
                {
                    Ok(Token::PseudoOp(ident.to_uppercase()))
                }
                // Check if it's an instruction
                else if self.is_instruction(&ident) {
                    Ok(Token::Instruction(ident.to_uppercase()))
                }
                // Otherwise it's an identifier
                else {
                    Ok(Token::Identifier(ident))
                }
            }

            Some(ch) => Err(AssemblerError::SyntaxError {
                line: self.line,
                message: format!("Unexpected character: '{}'", ch),
            }),
        }
    }

    /// Check if identifier is a valid instruction
    fn is_instruction(&self, ident: &str) -> bool {
        matches!(
            ident.to_uppercase().as_str(),
            "LD" | "LDD"
                | "STO"
                | "STD"
                | "A"
                | "AD"
                | "S"
                | "SD"
                | "M"
                | "D"
                | "AND"
                | "OR"
                | "EOR"
                | "SLA"
                | "SLCA"
                | "SRA"
                | "SRT"
                | "BSI"
                | "BC"
                | "BSC"
                | "LDX"
                | "STX"
                | "MDX"
                | "WAIT"
                | "LDS"
                | "STS"
                | "XIO"
                | "SDS"
        )
    }

    /// Tokenize entire source
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token, Token::Eof);
            tokens.push(token);
            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_instruction() {
        let source = "LD 100";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 3); // LD, 100, EOF
        assert_eq!(tokens[0], Token::Instruction("LD".to_string()));
        assert_eq!(tokens[1], Token::Number(100));
        assert_eq!(tokens[2], Token::Eof);
    }

    #[test]
    fn test_tokenize_with_label() {
        let source = "START LD 100";
        let mut lexer = Lexer::new(source);

        let token1 = lexer.next_token().unwrap();
        assert_eq!(token1, Token::Identifier("START".to_string()));

        let token2 = lexer.next_token().unwrap();
        assert_eq!(token2, Token::Instruction("LD".to_string()));

        let token3 = lexer.next_token().unwrap();
        assert_eq!(token3, Token::Number(100));
    }

    #[test]
    fn test_tokenize_hex_number() {
        let source = "0x1234";
        let mut lexer = Lexer::new(source);
        let token = lexer.next_token().unwrap();

        assert_eq!(token, Token::Number(0x1234));
    }

    #[test]
    fn test_tokenize_octal_number() {
        let source = "0777";
        let mut lexer = Lexer::new(source);
        let token = lexer.next_token().unwrap();

        assert_eq!(token, Token::Number(0o777));
    }

    #[test]
    fn test_tokenize_with_index() {
        let source = "100,1";
        let mut lexer = Lexer::new(source);

        assert_eq!(lexer.next_token().unwrap(), Token::Number(100));
        assert_eq!(lexer.next_token().unwrap(), Token::Comma);
        assert_eq!(lexer.next_token().unwrap(), Token::Number(1));
    }

    #[test]
    fn test_tokenize_indirect() {
        let source = "/100";
        let mut lexer = Lexer::new(source);

        assert_eq!(lexer.next_token().unwrap(), Token::Slash);
        assert_eq!(lexer.next_token().unwrap(), Token::Number(100));
    }

    #[test]
    fn test_tokenize_comment() {
        let source = "* This is a comment\nLD 100";
        let mut lexer = Lexer::new(source);

        assert_eq!(lexer.next_token().unwrap(), Token::Newline);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Instruction("LD".to_string())
        );
    }

    #[test]
    fn test_tokenize_pseudo_ops() {
        let source = "ORG 0x100\nDC 42\nEND";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0], Token::PseudoOp(_)));
        assert!(matches!(tokens[2], Token::Newline));
        assert!(matches!(tokens[3], Token::PseudoOp(_)));
    }
}
