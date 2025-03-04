use crate::enums::{Node, Token};
use crate::lexer::Lexer;
use crate::parser::parse_identifier::parse_identifier;
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

// parse expression ends on newline or on specified token. The exit token is returned
pub fn parse_expression(lexer: &mut Lexer, stop: &[TToken]) -> (Box<Node>, Token) {
    let mut output: Vec<Node> = Vec::new();
    let mut operators: Vec<Node> = Vec::new();
    // Previous token is plus or minus
    let mut last_token_is_pm = true;
    let exit_token;

    // Check if this is a call to create a new instance
    if let Some(Token {
        t: TToken::New,
        pos: _,
    }) = lexer.peek()
    {
        return create_object(lexer);
    }

    loop {
        match lexer.peek().unwrap().t {
            TToken::Caret | TToken::Identifier(_) => {
                output.push(*parse_identifier(lexer));
                last_token_is_pm = false;
            }
            _ => {
                let token = lexer.next().unwrap();
                if stop.contains(&token.t) {
                    exit_token = token.clone();
                    if output.is_empty() {
                        err("Expression expected", &token.pos);
                    }
                    break;
                }
                match token.t {
                    TToken::IntegerLit(val) => {
                        output.push(Node::Int {
                            val,
                            pos: token.pos,
                        });
                        last_token_is_pm = false;
                    }
                    TToken::RealLit(val) => {
                        output.push(Node::Real {
                            val,
                            pos: token.pos,
                        });
                        last_token_is_pm = false;
                    }
                    TToken::StringLit(val) => {
                        output.push(Node::String {
                            val,
                            pos: token.pos,
                        });
                        last_token_is_pm = false;
                    }
                    TToken::BoolLit(val) => {
                        output.push(Node::Boolean {
                            val,
                            pos: token.pos,
                        });
                        last_token_is_pm = false;
                    }
                    TToken::DateLit(val) => {
                        output.push(Node::Date {
                            val,
                            pos: token.pos,
                        });
                        last_token_is_pm = false;
                    }
                    TToken::Operator(ref op) => {
                        let mut _op = op.clone();

                        if last_token_is_pm && (op == "+" || op == "-") {
                            _op = format!("_{}", op);
                        }

                        last_token_is_pm = op == "+" || op == "-";

                        while let Some(top) = operators.last() {
                            if let Node::Op { op: top_op, pos: _ } = top.clone() {
                                if top_op == "(" {
                                    break;
                                }

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
                        operators.push(Node::Op {
                            op: _op,
                            pos: token.clone().pos,
                        });
                    }

                    TToken::LParen => {
                        // set to true as LParen can also be the first character
                        last_token_is_pm = true;
                        operators.push(Node::Op {
                            op: "(".to_string(),
                            pos: token.clone().pos,
                        });
                    }
                    TToken::RParen => {
                        last_token_is_pm = false;
                        while let Some(top) = operators.pop() {
                            match &top {
                                Node::Op { op, pos: _ } => {
                                    if op == "(" {
                                        break;
                                    }
                                    output.push(top);
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                    TToken::Newline | TToken::EOF => {
                        exit_token = token.clone();
                        if output.is_empty() {
                            err("Expression expected", &token.pos);
                        }
                        break;
                    }
                    _ => err("Invalid token", &token.pos),
                }
            }
        }
    }

    while let Some(node) = operators.pop() {
        match &node {
            Node::Op { op, pos } => {
                if op == "(" || op == ")" {
                    err("Mismatched parentheses", pos);
                } else {
                    output.push(node);
                }
            }
            _ => unreachable!(),
        }
    }

    (
        Box::from(Node::Expression(
            output
                .into_iter()
                .map(Box::from)
                .collect::<Vec<Box<Node>>>(),
        )),
        exit_token,
    )
}

fn create_object(lexer: &mut Lexer) -> (Box<Node>, Token) {
    // skip NEW token
    let token = lexer.next();
    let call = parse_identifier(lexer);
    match *call {
        Node::FunctionCall { .. } => (),
        _ => err("Class constructor call expected", &token.unwrap().pos),
    }
    (Box::from(Node::CreateObject(call)), lexer.next().unwrap())
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
        "&" => Operator {
            precedence: 4,
            associativity: Associativity::Left,
        },
        _ => unreachable!(),
    }
}
