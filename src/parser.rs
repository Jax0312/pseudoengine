use crate::enums::{Array, MOperator, VariableType};
use crate::tokens::*;
use crate::FILEPATH;
use codemap::CodeMap;
use logos::Lexer;
use std::fs::File;
use std::io::Read;

pub fn parse_file(lexer: & mut Lexer<Token>) -> Vec<Box<Node>> {
    let mut nodes= Vec::new();
    let mut main_children = Vec::<Box<Node>>::new();
    
    
    loop {
        match lexer.next() {
            Some(Ok(Token::Procedure)) => unimplemented!(),
            Some(Ok(Token::Function)) => unimplemented!(),
            Some(Ok(Token::Class)) => unimplemented!(),
            Some(Ok(token)) => main_children.push(parse_line(token, lexer)),
            _ => break,
        }
    }
    
    nodes.push(Box::from(Node::Main {children: main_children}));
    println!("{:?}", nodes);

    nodes
}

pub fn parse_line( token: Token, lexer: &mut Lexer<Token>) -> Box<Node> {
        match token {
            Token::Declare => parse_declare(lexer),
            _ => Box::new(Node::Null),
        }
}

fn parse_declare(lexer: &mut Lexer<Token>) -> Box<Node> {
    let mut expect_ident = false;
    let mut vars = Vec::<String>::new();
    let mut current;

    // Handle one or more identifier
    loop {
        current = lexer.next();
        match current {
            Some(Ok(Token::Identifier(name))) => {
                vars.push(name);
                expect_ident = false;
            }
            Some(Ok(Token::Comma)) => {
                if expect_ident {
                    err("Identifier expected", lexer);
                }
                expect_ident = true;
            }
            _ => break,
        }
    }

    if expect_ident {
        err("Identifier expected", lexer);
    } else if current.unwrap().unwrap() != Token::Colon{
        err(": expected", lexer);
    }
    

    // Handle variable type

    match lexer.next() {
        Some(Ok(Token::VarType(vt))) => Box::new(Node::Declare {
            t: Box::from(vt),
            children: vars,
        }),
        Some(Ok(Token::Array)) => Box::new(Node::Declare {
            t: parse_array(lexer),
            children: vars,
        }),
        _ => err("Type expected", lexer),
    }
}

fn parse_array(lexer: &mut Lexer<Token>) -> Box<VariableType> {

    expect_token(lexer, Token::LSqrBracket, "[");
    
    parse_array_dimension(lexer)
}

fn parse_array_dimension(lexer: &mut Lexer<Token>) -> Box<VariableType> {
    
    // The function calls itself recursively to create a nested array structure
    
    let mut v= Box::new(Array{t: Box::from(VariableType::Integer), lower: 0, upper: 0});
    
    if let Some(Token::IntegerLit(val)) = expect_token(lexer, Token::IntegerLit(0), "Integer") { v.lower = val }
    
    expect_token(lexer, Token::Colon, ":");
    
    if let Some(Token::IntegerLit(val)) = expect_token(lexer, Token::IntegerLit(0), "Integer") { v.upper = val }
    
    match lexer.next() {
        Some(Ok(Token::Comma)) => {
            Box::from(VariableType::Array( Box::from(Array {
                t: parse_array_dimension(lexer),
                lower: v.lower,
                upper: v.upper,
            })))
        }
        Some(Ok(Token::RSqrBracket)) => {
            expect_token(lexer, Token::Of, "'Of'");
            if let Some(Ok(Token::VarType(vt))) = lexer.next() {
                Box::from(VariableType::Array( Box::from(Array {
                    t: Box::from(vt),
                    lower: v.lower,
                    upper: v.upper,
                })))
            } else {
                err("Type expected", lexer);
            }
        },
        _ => err(" ']' or ',' expected", lexer),
    }
}

fn expect_token(
    lexer: &mut Lexer<Token>,
    token: Token,
    message: &str,
) -> Option<Token> {
    let next = lexer.next().unwrap().unwrap();
    if std::mem::discriminant(&next) == std::mem::discriminant(&token) {
        Some(next)
    } else {
        println!("{:?}", next);
        err(&format!("{} expected", message), lexer);
    }
}
fn err(message: &str, lexer: &mut Lexer<Token>) -> ! {
    let mut cm = CodeMap::new();
    let mut source = File::open(FILEPATH).unwrap();
    let mut buf = String::new();
    source.read_to_string(&mut buf).unwrap();
    let file = cm.add_file("source".to_string(), buf);
    let location = cm.look_up_span(
        file.span
            .subspan(lexer.span().start as u64, lexer.span().end as u64),
    );
    println!(
        "{} at line {} col {}",
        message,
        location.begin.line + 1,
        location.begin.column
    );
    panic!()
}

// pub fn parse(tokens: &mut [Token]) -> Vec<Box<Node>> {
//     let mut nodes = Vec::<Box<Node>>::new();
//
//     let mut tokens = tokens.iter().peekable();
//
//     while tokens.peek().is_some() {
//         match &tokens.next().unwrap().t_type {
//             TokenType::Declare => nodes.push(parse_declare(&mut tokens)),
//             TokenType::Output => nodes.push(parse_output(&mut tokens)),
//             TokenType::Input => nodes.push(parse_input(&mut tokens)),
//             TokenType::Identifier(identifier) => {
//                 nodes.push(parse_assignment(&mut tokens, identifier.clone()))
//             }
//             _ => (),
//         }
//     }
//
//     println!("{:?}", nodes);
//
//     nodes
// }
//
// fn parse_assignment(tokens: &mut Peekable<Iter<Token>>, identifier: String) -> Box<Node> {
//     let token = tokens.next();
//     if token.is_none() || token.unwrap().t_type != TokenType::Assignment {
//         ()
//     }
//
//     // TODO
//     // So currently the op pred parsing can only be triggered by an expression starting with num lit.
//     // It should also trigger if the leading token of rhs is a identifier
//     // More robust handling required
//
//     Box::new(Node::Assignment {
//         lhs: Box::new(Node::Var(identifier.clone())),
//         rhs: Box::new(parse_expr(tokens)),
//     })
// }
//
// fn parse_expr(tokens: &mut Peekable<Iter<Token>>) -> Node {
//     let lhs = parse_primary(tokens);
//     parse_expr_precedence(tokens, lhs, 0)
// }
//
// fn parse_expr_precedence(
//     tokens: &mut Peekable<Iter<Token>>,
//     mut lhs: Node,
//     precedence: i8,
// ) -> Node {
//     if let Some(token) = tokens.peek() {
//         let mut lookahead = (*token).clone();
//         while tokens.peek().is_some() && op_precedence(&lookahead.t_type) >= precedence {
//             let op = lookahead;
//             if tokens.next().is_some() {
//                 let mut rhs = parse_primary(tokens);
//                 if let Some(token) = tokens.peek() {
//                     lookahead = (*token).clone();
//                     while tokens.peek().is_some()
//                         && op_precedence(&lookahead.t_type) > op_precedence(&op.t_type)
//                     {
//                         rhs = parse_expr_precedence(tokens, rhs, op_precedence(&lookahead.t_type));
//                         if let Some(token) = tokens.peek() {
//                             lookahead = (*token).clone();
//                         }
//                     }
//                 }
//                 lhs = Node::BinaryExpr {
//                     op: Operator::from(&op.t_type),
//                     lhs: Box::new(lhs),
//                     rhs: Box::new(rhs),
//                 };
//             }
//         }
//     }
//     lhs
// }
//
// fn parse_primary(tokens: &mut Peekable<Iter<Token>>) -> Node {
//     let token = tokens.next();
//
//     if token.is_none() {
//         eprintln!(
//             "Expected expr at line {:?}:{:?}",
//             &token.unwrap().line_c,
//             &token.unwrap().col_s
//         );
//         panic!();
//     }
//
//     match &token.unwrap().t_type {
//         TokenType::NumLit(lit) => Node::Int(*lit),
//         TokenType::StringLit(lit) => Node::String(lit.clone()),
//         TokenType::Identifier(name) => Node::Var(name.clone()),
//         _ => unimplemented!(),
//     }
// }
//
// fn parse_input(tokens: &mut Peekable<Iter<Token>>) -> Box<Node> {
//     let token = tokens.next();
//     if token.is_none() {
//         eprintln!(
//             "Expected identifier at line {:?}:{:?}",
//             &token.unwrap().line_c,
//             &token.unwrap().col_s
//         );
//         panic!();
//     }
//
//     if let TokenType::Identifier(name) = &token.unwrap().t_type {
//         Box::new(Node::Input {
//             child: Box::new(Node::Var(name.clone())),
//         })
//     } else {
//         eprintln!(
//             "Expected identifier at line {:?}:{:?}",
//             &token.unwrap().line_c,
//             &token.unwrap().col_s
//         );
//         panic!();
//     }
// }
//
// fn parse_output(tokens: &mut Peekable<Iter<Token>>) -> Box<Node> {
//     let mut children = Vec::<Box<Node>>::new();
//     let mut expect_separator = false;
//     loop {
//         let token = tokens.peek();
//         if token.is_none() || token.unwrap().t_type == TokenType::LineEnd {
//             break;
//         }
//         if expect_separator {
//             if token.unwrap().t_type != TokenType::Comma {
//                 break;
//             }
//             tokens.next();
//             expect_separator = false;
//         } else {
//             children.push(Box::new(parse_expr(tokens)));
//             expect_separator = true;
//         }
//     }
//     Box::new(Node::Output { children })
// }
//
// fn parse_declare(tokens: &mut Peekable<Iter<Token>>) -> Box<Node> {
//     let mut identifiers = Vec::<String>::new();
//
//     loop {
//         let mut token = tokens.next();
//         if token.is_none() {
//             break;
//         }
//         match &token.unwrap().t_type {
//             TokenType::Identifier(name) => {
//                 identifiers.push(name.clone());
//                 token = tokens.next();
//                 match &token.unwrap().t_type {
//                     TokenType::Comma => (),
//                     TokenType::Colon => break,
//                     TokenType::Identifier(_) => {
//                         eprintln!(
//                             "Expected comma at line {:?}:{:?}",
//                             &token.unwrap().line_c,
//                             &token.unwrap().col_s
//                         );
//                         panic!();
//                     },
//                     _ => {
//                         eprintln!(
//                             "Error declaring at line {:?}",
//                             &token.unwrap().line_c,
//                         );
//                         panic!();
//                     },
//                 }
//             }
//             _ => {
//                 eprintln!(
//                     "Expected identifier at line {:?}:{:?}",
//                     &token.unwrap().line_c,
//                     &token.unwrap().col_s
//                 );
//                 panic!();
//             }
//         }
//     }
//
//     let token = tokens.next();
//     match &token.unwrap().t_type {
//         TokenType::Integer => Box::new(Node::Declare {
//             t: VariableType::Integer,
//             children: identifiers,
//         }),
//         TokenType::Real => Box::new(Node::Declare {
//             t: VariableType::Real,
//             children: identifiers,
//         }),
//         TokenType::Char => Box::new(Node::Declare {
//             t: VariableType::Char,
//             children: identifiers,
//         }),
//         TokenType::String => Box::new(Node::Declare {
//             t: VariableType::String,
//             children: identifiers,
//         }),
//         TokenType::Date => Box::new(Node::Declare {
//             t: VariableType::Date,
//             children: identifiers,
//         }),
//         _ => {
//             eprintln!("Expected a valid primitive type");
//             panic!()
//         }
//     }
// }
//
// #[derive(Debug, Clone)]
// pub enum Operator {
//     Plus,
//     Minus,
//     Star,
//     Slash,
//     Equal,
//     NotEqual,
//     Greater,
//     Lesser,
//     GreaterEqual,
//     LesserEqual,
//     And,
//     Or,
//     Not,
// }
//
// impl From<&TokenType> for Operator {
//     fn from(t_type: &TokenType) -> Operator {
//         match t_type {
//             TokenType::Plus => Operator::Plus,
//             TokenType::Minus => Operator::Minus,
//             TokenType::Star => Operator::Star,
//             TokenType::Slash => Operator::Slash,
//             TokenType::Equal => Operator::Equal,
//             TokenType::NotEqual => Operator::NotEqual,
//             TokenType::Greater => Operator::Greater,
//             TokenType::Lesser => Operator::Lesser,
//             TokenType::GreaterEqual => Operator::GreaterEqual,
//             TokenType::LesserEqual => Operator::LesserEqual,
//             TokenType::And => Operator::And,
//             TokenType::Or => Operator::Or,
//             TokenType::Not => Operator::Not,
//             _ => unreachable!(),
//         }
//     }
// }
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Node {
    Main {
        children: Vec<Box<Node>>,
    },
    Var(String),
    Int(i64),
    String(String),
    Declare {
        t: Box<VariableType>,
        // Identifiers
        children: Vec<String>,
    },
    Assignment {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    BinaryExpr {
        op: MOperator,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Output {
        children: Vec<Box<Node>>,
    },
    Input {
        child: Box<Node>,
    },
    Null,
}
//
// fn op_precedence(t_type: &TokenType) -> i8 {
//     match t_type {
//         TokenType::Plus => 5,
//         TokenType::Minus => 5,
//         TokenType::Star => 6,
//         TokenType::Slash => 6,
//         TokenType::Equal => 3,
//         TokenType::NotEqual => 3,
//         TokenType::Greater => 4,
//         TokenType::Lesser => 4,
//         TokenType::GreaterEqual => 4,
//         TokenType::LesserEqual => 4,
//         TokenType::And => 2,
//         TokenType::Or => 1,
//         TokenType::Not => 1,
//         _ => -1,
//     }
// }
