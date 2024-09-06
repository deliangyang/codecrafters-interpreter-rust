use std::fmt::Display;



#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    VAR,
    AND,
    CLASS,
    ELSE,
    FALSE,
    FOR,
    FUN,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    WHILE,
    IDENTIFIER(String),
    EQUAL,
    STRING(String),
    Number(String),
    SEMICOLON, // ;
    EOF,       // null

    LeftParen,  // (
    RightParen, // )

    LeftBrace,  // {
    RightBrace, // }
    // ,, ., -, +, ; and *. /
    Star,  // *
    Slash, // /
    Minus, // -
    Plus,  // +
    Comma, // ,
    Dot,   // .

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
            Token::VAR => write!(f, "VAR var null"),
            Token::AND => write!(f, "AND and null"),
            Token::CLASS => write!(f, "CLASS class null"),
            Token::ELSE => write!(f, "ELSE else null"),
            Token::FALSE => write!(f, "FALSE false null"),
            Token::FOR => write!(f, "FOR for null"),
            Token::FUN => write!(f, "FUN fun null"),
            Token::IF => write!(f, "IF if null"),
            Token::NIL => write!(f, "NIL nil null"),
            Token::OR => write!(f, "OR or null"),
            Token::PRINT => write!(f, "PRINT print null"),
            Token::RETURN => write!(f, "RETURN return null"),
            Token::SUPER => write!(f, "SUPER super null"),
            Token::THIS => write!(f, "THIS this null"),
            Token::TRUE => write!(f, "TRUE true null"),
            Token::WHILE => write!(f, "WHILE while null"),
            Token::IDENTIFIER(s) => write!(f, "IDENTIFIER {} null", s),
            Token::EQUAL => write!(f, "EQUAL = null"),
            Token::STRING(s) => write!(f, "STRING \"{}\" {}", s, s),
            Token::SEMICOLON => write!(f, "SEMICOLON ; null"),
            Token::EOF => write!(f, "EOF  null"),
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
