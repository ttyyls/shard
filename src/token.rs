use crate::location::Location;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub enum TokenKind {
    Ampersand,
    At,
    Backslash,
    Bang,
    Caret,
    CharLiteral,
    Colon,
    Comma,
    Dollar,
    Dot,
    DoubleQuote,
    EOF,
    Equals,
    FatArrow,
    FloatLiteral,
    GreaterThan,
    GreaterThanEquals,
    Identifier,
    IntegerLiteral,
    Jmp,
    LeftBrace,
    LeftBracket,
    LeftParen,
    LessThan,
    LessThanEquals,
    Minus,
    MinusMinus,
    Newline,
    NotEquals,
    Percent,
    Pipe,
    Plus,
    PlusPlus,
    Pound,
    Question,
    Register,
    Ret,
    RightBrace,
    RightBracket,
    RightParen,
    Semicolon,
    SingleQuote,
    Slash,
    Star,
    StringLiteral,
    Tilde,
    TinyArrowLeft,
    TinyArrowRight,
    Underscore,
}

pub struct Token {
    pub kind: TokenKind,
    pub location: Location,
    pub text: String,
    pub flag: u8,
    /*
        1-3: Register size
     */
}

impl Token {
    pub fn new(kind: TokenKind, location: Location, text: String) -> Token {
        Token {
            kind,
            location,
            text,
            flag: 0,
        }
    }

    pub fn new_simple(kind: TokenKind, location: Location) -> Token {
        Token::new(kind, location, String::new())
    }

    pub fn new_eof(location: Location) -> Token {
        Token::new_simple(TokenKind::EOF, location)
    }

    pub fn from_string(location: Location, text: String) -> Token {
        Token {
            kind: match text.as_ref() {
                "ret" => TokenKind::Ret,
                "jmp" => TokenKind::Jmp,
                _ => TokenKind::Identifier,
            },
            location,
            text,
            flag: 0,
        }
    }

    pub fn register_size(&self) -> u8 {
        self.flag & 0b0000_0111
    }

    pub fn newline_before(self) -> bool {
        self.flag & 0b1000_0000 != 0
    }

    pub fn set_register_size(&mut self, size: u8) {
        self.flag &= 0b1111_1000;
        self.flag |= size & 0b0000_0111;
    }

    pub fn set_flag_bit(&mut self, bit: u8, val: bool) {
        if val {
            self.flag |= 1 << bit;
        } else {
            self.flag &= !(1 << bit);
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.kind)?;
        if !self.text.is_empty() {
            if self.flag != 0 {
                write!(f, "({:?}, f{})", self.text, self.flag)?;
            } else {
                write!(f, "({:?})", self.text)?;
            }
        }
        Ok(())
    }
}
