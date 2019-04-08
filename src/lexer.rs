#[derive(Eq, PartialEq, Clone, Debug)]
pub enum KwdType {
    Int,      // int
    Void,     // void
    Ret,      // return
    If,       // if
    Else,     // else
    While,    // while
    For,      // for
    Do,       // do
    Break,    // break
    Continue, // continue
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum TokType {
    Kwd(KwdType),
    LBrace,             // {
    RBrace,             // }
    LParen,             // (
    RParen,             // )
    LBracket,           // [
    RBracket,           // ]
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
    Comma,              // ,
    String(String, String),
    Addr,               // &var
}

static mut LABEL_COUNTER: i64 = -1;
fn gen_string_tag() -> String {
    unsafe {
        LABEL_COUNTER = LABEL_COUNTER + 1;
        return format!(".LSTR{}", LABEL_COUNTER);
    }
}
pub fn lex(input: &str) -> Result<Vec<TokType>, String> {
    let mut result = Vec::new();

    let mut it = input.chars().peekable();

    while let Some(&c) = it.peek() {
        match c {
            '"' => {
                it.next();
                let mut s = "".to_string();
                while let &c = it.peek().unwrap() {
                    if (c == '"') {
                        break;
                    }
                    s.push(c);
                    it.next();
                }
                result.push(TokType::String(s, gen_string_tag()));
                it.next();
            }
            '\'' => {
                // try parse a char
                // now just use int to represent char
                // transform it to int
                it.next(); // skip '
                let &c = it.peek().unwrap();
                if (c == '\'') {
                    return Err(format!("Error: empty character constant"));
                }
                if (c == '\\') {
                    it.next();
                    let &c = it.peek().unwrap();
                    match c {
                        'a' => {
                            result.push(TokType::Literal(0x07));
                        } // Alert (Beep, Bell) (added in C89)
                        'b' => {
                            result.push(TokType::Literal(0x08));
                        } // Backspace
                        'e' => {
                            result.push(TokType::Literal(0x1B));
                        } // escape character
                        'f' => {
                            result.push(TokType::Literal(0x0C));
                        } // Formfeed Page Break
                        'n' => {
                            result.push(TokType::Literal(0x0A));
                        } // Newline (Line Feed)
                        'r' => {
                            result.push(TokType::Literal(0x0D));
                        } // Carriage Return
                        't' => {
                            result.push(TokType::Literal(0x09));
                        } // Horizontal Tab
                        'v' => {
                            result.push(TokType::Literal(0x0B));
                        } // Vertical Tab
                        '\\' => {
                            result.push(TokType::Literal(0x5C));
                        } // Backslash
                        '\'' => {
                            result.push(TokType::Literal(0x27));
                        } // Apostrophe or single quotation mark
                        '\"' => {
                            result.push(TokType::Literal(0x22));
                        } // Double quotation mark
                        '?' => {
                            result.push(TokType::Literal(0x3F));
                        } // question mark
                        _ => {
                            return Err(format!("unrecongnized character"));
                        }
                    }
                    it.next();
                    if it.peek().unwrap() != &'\'' {
                        return Err(format!("Error: unmatched '"));
                    }
                    it.next();
                } else {
                    result.push(TokType::Literal(c as i64));
                    it.next(); // skip char
                    it.next(); // skip '
                }
            }
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
                    "char" => result.push(TokType::Kwd(KwdType::Int)),
                    "return" => result.push(TokType::Kwd(KwdType::Ret)),
                    "void" => result.push(TokType::Kwd(KwdType::Void)),
                    "if" => result.push(TokType::Kwd(KwdType::If)),
                    "else" => result.push(TokType::Kwd(KwdType::Else)),
                    "while" => result.push(TokType::Kwd(KwdType::While)),
                    "for" => result.push(TokType::Kwd(KwdType::For)),
                    "do" => result.push(TokType::Kwd(KwdType::Do)),
                    "continue" => result.push(TokType::Kwd(KwdType::Continue)),
                    "break" => result.push(TokType::Kwd(KwdType::Break)),
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
            '[' => {
                result.push(TokType::LBracket);
                it.next();
            }
            ']' => {
                result.push(TokType::RBracket);
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
                            // & operator to get the address of a variable
                            result.push(TokType::Addr);
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
            ',' => {
                result.push(TokType::Comma);
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
