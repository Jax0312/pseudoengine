use crate::enums::{Position, Token};
use crate::lexer::Lexer;
use crate::tokens::TToken;

pub const SUPPORT_MESSAGE: &str = "\nIf you believe this behaviour is not in line with Cambridge A-Lvls standard, please email jaxonmoh05@gmail.com";

// Match the next token against a list of expected tokens
pub fn expect_token(
    lexer: &mut Lexer,
    tokens: &[TToken],
    message: &str,
) -> Token {
    
    if let Some(next) = lexer.next() {
        if next.t == TToken::EOF {
            err(&format!("{} expected", message), &next.pos);        
        }
        for token in tokens {
            if std::mem::discriminant(&next.t) == std::mem::discriminant(token) {
                return next;
            }
        }
        err(&format!("{} expected", message), &next.pos);
    }
    
    unreachable!()
    
}

// Parser error function, contains position metadata
pub fn err(message: &str, pos: &Position) -> ! {
    println!("{} at line {} col {}", message, pos.line, pos.col);
    panic!()
}
