#[derive(Eq, PartialEq, Clone, Debug)]
pub enum KwdType {
    Int,   // int
    Void,  // void
    Ret,   // return
    If,    // if
    Else,  // else
    While, // while
    For,   // for
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum TokType {
    Kwd(KwdType),
    LBrace,             // {
    RBrace,             // }
    LParen,             // (
    RParen,             // )
    Semicolon,          // ;
    Assign,             // =
    Lt,                 // <
    Gt,                 // >
    Minus,              // -
    Tilde,              // ~
    Exclamation,        // !
    Plus,               // +
    Multi,              // *
    Splash,             // /
    Literal(i64),       // [0-9]+
    Identifier(String), // identifier
    And,                // &&
    Or,                 // ||
    Equal,              // ==
    NotEqual,           // !=
    LessEqual,          // <=
    GreaterEqual,       // >=
    Colon,              // :
    QuestionMark,       // ?
}

pub fn lex(input: &String) -> Result<Vec<TokType>, String> {
    let mut result = Vec::new();

    let mut it = input.chars().peekable();

    while let Some(&c) = it.peek() {
        match c {
            '0'...'9' => {
                it.next();
                let mut number = c
                    .to_string()
                    .parse::<i64>()
                    .expect("The caller should have passed a digit.");

                while let Some(Ok(digit)) = it.peek().map(|c| c.to_string().parse::<i64>()) {
                    number = number * 10 + digit;
                    it.next();
                }
                result.push(TokType::Literal(number));
            }
            'a'...'z' | 'A'...'Z' | '_' => {
                it.next();
                let mut s = String::new();
                s.push(c);
                while let Some(&tmp) = it.peek() {
                    match tmp {
                        'a'...'z' | 'A'...'Z' | '_' => {
                            s.push(tmp);
                            it.next();
                        }
                        _ => {
                            break;
                        }
                    }
                }
                match s.as_ref() {
                    "int" => result.push(TokType::Kwd(KwdType::Int)),
                    "return" => result.push(TokType::Kwd(KwdType::Ret)),
                    "void" => result.push(TokType::Kwd(KwdType::Void)),
                    "if" => result.push(TokType::Kwd(KwdType::If)),
                    "else" => result.push(TokType::Kwd(KwdType::Else)),
                    "while" => result.push(TokType::Kwd(KwdType::While)),
                    "for" => result.push(TokType::Kwd(KwdType::For)),
                    _ => result.push(TokType::Identifier(s)),
                }
            }
            '(' => {
                result.push(TokType::LParen);
                it.next();
            }
            ')' => {
                result.push(TokType::RParen);
                it.next();
            }
            '{' => {
                result.push(TokType::LBrace);
                it.next();
            }
            '}' => {
                result.push(TokType::RBrace);
                it.next();
            }
            ';' => {
                result.push(TokType::Semicolon);
                it.next();
            }
            '=' => {
                it.next();
                match it.peek() {
                    Some(tmp) => match tmp {
                        '=' => {
                            result.push(TokType::Equal);
                            it.next();
                        }
                        '>' => {
                            result.push(TokType::GreaterEqual);
                            it.next();
                        }
                        _ => {
                            result.push(TokType::Assign);
                        }
                    },
                    _ => return Err(format!("Can not peek next char")),
                }
            }
            '<' => {
                it.next();
                match it.peek() {
                    Some(tmp) => match tmp {
                        '=' => {
                            it.next();
                            result.push(TokType::LessEqual);
                            it.next();
                        }
                        _ => {
                            result.push(TokType::Lt);
                            it.next();
                        }
                    },
                    _ => return Err(format!("Can not peek next char")),
                }
            }
            '>' => {
                it.next();
                match it.peek() {
                    Some(tmp) => match tmp {
                        '=' => {
                            result.push(TokType::GreaterEqual);
                            it.next();
                        }
                        _ => {
                            result.push(TokType::Gt);
                            it.next();
                        }
                    },
                    _ => return Err(format!("Can not peek next char")),
                }
            }
            '-' => {
                result.push(TokType::Minus);
                it.next();
            }
            '~' => {
                result.push(TokType::Tilde);
                it.next();
            }
            '!' => {
                it.next();
                match it.peek() {
                    Some(tmp) => match tmp {
                        '=' => {
                            result.push(TokType::NotEqual);
                            it.next();
                        }
                        _ => {
                            result.push(TokType::Exclamation);
                        }
                    },
                    _ => return Err(format!("Can not peek next char")),
                }
            }
            '+' => {
                result.push(TokType::Plus);
                it.next();
            }
            '*' => {
                result.push(TokType::Multi);
                it.next();
            }
            '/' => {
                result.push(TokType::Splash);
                it.next();
            }
            '&' => {
                it.next();
                match it.peek() {
                    Some(tmp) => match tmp {
                        '&' => {
                            result.push(TokType::And);
                            it.next();
                        }
                        _ => {
                            // now don't support bitwise and, so just return Err
                            return Err(format!("unexpected token &{}", c));
                        }
                    },
                    _ => return Err(format!("Can not peek next char")),
                }
            }
            '|' => {
                it.next();
                match it.peek() {
                    Some(tmp) => match tmp {
                        '|' => {
                            result.push(TokType::Or);
                            it.next();
                        }
                        _ => {
                            // now don't support bitwise or, so just return Err
                            return Err(format!("unexpected token {}", c));
                        }
                    },
                    _ => return Err(format!("Can not peek next char")),
                }
            }
            '?' => {
                result.push(TokType::QuestionMark);
                it.next();
            }
            ':' => {
                result.push(TokType::Colon);
                it.next();
            }
            ' ' | '\n' | '\t' | '\r' => {
                // skip
                it.next();
            }
            _ => {
                return Err(format!("unexpected character {}", c));
            }
        }
    }
    Ok(result)
}
