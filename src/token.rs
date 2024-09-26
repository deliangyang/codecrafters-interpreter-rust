use std::fmt::Display;



#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Var,
    And,
    Class,
    New,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    While,
    Switch,
    Case,
    Default,
    Identifier(String),
    Equal,
    String(String),
    Number(String),
    Semicolon, // ;
    Eof,       // null

    LeftParen,  // (
    RightParen, // )

    LeftBrace,  // {
    RightBrace, // }

    LeftBracket,  // [
    RightBracket, // ]

    // ,, ., -, +, ; and *. /
    Star,  // *
    Slash, // /
    Minus, // -
    Plus,  // +
    Comma, // ,
    Dot,   // .
    Colon, // :

    EqualEqual, // ==
    Bang,       // !
    BangEqual,  // !=

    Less,      // <
    LessEqual, // <=

    Greater,      // >
    GreaterEqual, // >=

    Comment(String), // //
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Var => write!(f, "VAR var null"),
            Token::And => write!(f, "AND and null"),
            Token::Class => write!(f, "CLASS class null"),
            Token::Else => write!(f, "ELSE else null"),
            Token::False => write!(f, "FALSE false null"),
            Token::For => write!(f, "FOR for null"),
            Token::Fun => write!(f, "FUN fun null"),
            Token::If => write!(f, "IF if null"),
            Token::Nil => write!(f, "NIL nil null"),
            Token::Or => write!(f, "OR or null"),
            Token::Print => write!(f, "PRINT print null"),
            Token::Return => write!(f, "RETURN return null"),
            Token::Super => write!(f, "SUPER super null"),
            Token::This => write!(f, "THIS this null"),
            Token::True => write!(f, "TRUE true null"),
            Token::While => write!(f, "WHILE while null"),
            Token::Identifier(s) => write!(f, "IDENTIFIER {} null", s),
            Token::Equal => write!(f, "EQUAL = null"),
            Token::String(s) => write!(f, "STRING \"{}\" {}", s, s),
            Token::Semicolon => write!(f, "SEMICOLON ; null"),
            Token::Eof => write!(f, "EOF  null"),
            Token::LeftParen => write!(f, "LEFT_PAREN ( null"),
            Token::RightParen => write!(f, "RIGHT_PAREN ) null"),
            Token::LeftBrace => write!(f, "LEFT_BRACE {{ null"),
            Token::RightBrace => write!(f, "RIGHT_BRACE }} null"),
            Token::Star => write!(f, "STAR * null"),
            Token::Slash => write!(f, "SLASH / null"),
            Token::Minus => write!(f, "MINUS - null"),
            Token::Plus => write!(f, "PLUS + null"),
            Token::Comma => write!(f, "COMMA , null"),
            Token::Dot => write!(f, "DOT . null"),
            Token::EqualEqual => write!(f, "EQUAL_EQUAL == null"),
            Token::Bang => write!(f, "BANG ! null"),
            Token::BangEqual => write!(f, "BANG_EQUAL != null"),
            Token::Less => write!(f, "LESS < null"),
            Token::LessEqual => write!(f, "LESS_EQUAL <= null"),
            Token::Greater => write!(f, "GREATER > null"),
            Token::GreaterEqual => write!(f, "GREATER_EQUAL >= null"),
            Token::Comment(s) => write!(f, "COMMENT //{} null", s),
            Token::Switch => write!(f, "SWITCH switch null"),
            Token::Case => write!(f, "CASE case null"),
            Token::Default => write!(f, "DEFAULT default null"),
            Token::Colon => write!(f, "COLON : null"),
            Token::New => write!(f, "NEW new null"),
            Token::LeftBracket => write!(f, "LEFT_BRACKET [ null"),
            Token::RightBracket => write!(f, "RIGHT_BRACKET ] null"),
            Token::Number(n) => {
                let num = n.parse::<f64>().unwrap();
                let inum = (num as i64) as f64;
                if num > inum {
                    write!(f, "NUMBER {} {}", n, num)
                } else {
                    write!(f, "NUMBER {} {}.0", n, num)
                }
            }
        }
    }
}
