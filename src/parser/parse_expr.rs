use crate::enums::{Node, Token};
use crate::lexer::Lexer;
use crate::tokens::TToken;
use crate::utils::err;

#[derive(Debug, PartialEq, Clone)]
enum Associativity {
    Left,
    Right,
}

struct Operator {
    precedence: u8,
    associativity: Associativity,
}

pub fn parse_expression(lexer: &mut Lexer) -> Box<Node> {
    let mut output: Vec<Token> = Vec::new();
    let mut operators: Vec<Token> = Vec::new();
    // Previous token is plus or minus
    let mut last_token_is_pm = true;

    loop {
        let token = lexer.next().unwrap();
        match token.t {
            TToken::IntegerLit(_)
            | TToken::BoolLit(_)
            | TToken::RealLit(_)
            | TToken::Identifier(_) => {
                output.push(token.clone());
                last_token_is_pm = false;
            }
            TToken::Operator(ref op) => {
                let mut _op = op.clone();

                if last_token_is_pm && (op == "+" || op == "-") {
                    _op = format!("_{}", op);
                }

                last_token_is_pm = op == "+" || op == "-";

                while let Some(top) = operators.last() {
                    if let TToken::Operator(top_op) = top.clone().t {
                        let top_op_info = get_operator_precedence(&top_op);
                        let op_info = get_operator_precedence(&_op);

                        if (op_info.associativity == Associativity::Left
                            && op_info.precedence <= top_op_info.precedence)
                            || (op_info.associativity == Associativity::Right
                                && op_info.precedence < top_op_info.precedence)
                        {
                            output.push(operators.pop().unwrap());
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                operators.push(Token {
                    t: TToken::Operator(_op),
                    pos: token.clone().pos,
                });
            }

            TToken::LParen => {
                last_token_is_pm = false;
                operators.push(Token {
                    t: TToken::LParen,
                    pos: token.clone().pos,
                });
            }
            TToken::RParen => {
                last_token_is_pm = false;
                while let Some(top) = operators.pop() {
                    if top.t == TToken::LParen {
                        break;
                    } else {
                        output.push(top);
                    }
                }
            }
            TToken::Newline => break,
            _ => err("Invalid token", &token.pos),
        }
    }

    while let Some(op) = operators.pop() {
        if op.t == TToken::LParen || op.t == TToken::RParen {
            err("Mismatched parentheses", &op.pos);
        }
        output.push(op);
    }

    Box::from(Node::Expression(output))
}

fn get_operator_precedence(op: &String) -> Operator {
    match op.as_str() {
        "&&" => Operator {
            precedence: 1,
            associativity: Associativity::Left,
        },
        "||" => Operator {
            precedence: 1,
            associativity: Associativity::Left,
        },
        "!" => Operator {
            precedence: 2,
            associativity: Associativity::Left,
        },
        "=" => Operator {
            precedence: 2,
            associativity: Associativity::Left,
        },
        "!=" => Operator {
            precedence: 2,
            associativity: Associativity::Left,
        },
        ">=" => Operator {
            precedence: 2,
            associativity: Associativity::Left,
        },
        ">" => Operator {
            precedence: 2,
            associativity: Associativity::Left,
        },
        "<" => Operator {
            precedence: 2,
            associativity: Associativity::Left,
        },
        "<=" => Operator {
            precedence: 2,
            associativity: Associativity::Left,
        },
        "+" => Operator {
            precedence: 3,
            associativity: Associativity::Left,
        },
        "-" => Operator {
            precedence: 3,
            associativity: Associativity::Left,
        },
        "_+" => Operator {
            precedence: 3,
            associativity: Associativity::Right,
        },
        "_-" => Operator {
            precedence: 3,
            associativity: Associativity::Right,
        },
        "*" => Operator {
            precedence: 4,
            associativity: Associativity::Left,
        },
        "/" => Operator {
            precedence: 4,
            associativity: Associativity::Left,
        },
        "//" => Operator {
            precedence: 4,
            associativity: Associativity::Left,
        },
        "%" => Operator {
            precedence: 4,
            associativity: Associativity::Left,
        },
        _ => unreachable!(),
    }
}
