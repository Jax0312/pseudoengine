use std::ops::Deref;

use annotate_snippets::{Level, Renderer, Snippet};

use crate::enums::{Position, Token};
use crate::lexer::Lexer;
use crate::tokens::TToken;
use crate::SOURCE_FILE;

pub const SUPPORT_MESSAGE: &str = "\nIf you believe this behaviour is not in line with Cambridge A-Lvls standard, please email jaxonmoh05@gmail.com";

// Match the next token against a list of expected tokens
pub fn expect_token(lexer: &mut Lexer, tokens: &[TToken], message: &str) -> Token {
    if let Some(next) = lexer.next() {
        if next.t == TToken::EOF {
            err(&format!("{} expected", message), &next.pos);
        }
        for token in tokens {
            if std::mem::discriminant(&next.t) == std::mem::discriminant(token) {
                return next;
            }
        }
        err(
            &format!("{} expected. {:?} found", message, next),
            &next.pos,
        );
    }

    unreachable!()
}

// Parser error function, contains position metadata
pub fn err(message: &str, pos: &Position) -> ! {
    let source = SOURCE_FILE.with(|file| file.borrow().deref().clone());
    let mut lines = source.file[pos.line_start - 1].clone();
    let mut start = pos.pos_start;
    let mut end = pos.pos_end;
    if pos.line_start != pos.line_end {
        let slice = &source.file[(pos.line_start - 1)..pos.line_end];
        lines = slice.join("\n");
        let mut len = slice[0].len() - pos.pos_start + 1;
        if pos.line_end - pos.line_start > 1 {
            for line in &slice[1..slice.len() - 1] {
                len += line.len() + 1;
            }
        }
        len += pos.pos_end;
        end = start + len;
    }
    let message = Level::Error.title(message).snippet(
        Snippet::source(&lines)
            .line_start(pos.line_start)
            .origin(&source.name)
            .fold(true)
            .annotation(Level::Error.span(start..end).label(message)),
    );

    let renderer = Renderer::styled();
    anstream::println!("{}", renderer.render(message));
    std::process::exit(0);
}
