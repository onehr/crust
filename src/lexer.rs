//     Copyright 2019 Haoran Wang
//
//     Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
//     You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
//     distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//     See the License for the specific language governing permissions and
//     limitations under the License.
// ------------------------------------------------------------------------
// lexer.rs: lexer for c11 tokens.
//           some situations should be added later.
// ------------------------------------------------------------------------
// TODO: 1. add token information for error message.
//       2. seperate each TokType to their type, now just a global type TokType.
//       3. add some check in lexer for enum and typedef.
//       4. add floating point number support.
//       5. number with postfix.

#[allow(dead_code)]
#[derive(PartialEq, Clone, Debug)]
pub enum TokType {
    LBrace,       // {
    RBrace,       // }
    LParen,       // (
    RParen,       // )
    LBracket,     // [
    RBracket,     // ]
    Semicolon,    // ;
    Assign,       // =
    Lt,           // <
    Gt,           // >
    Minus,        // -
    Tilde,        // ~
    Exclamation,  // !
    Plus,         // +
    Multi,        // *
    Splash,       // /
    Colon,        // :
    QuestionMark, // ?
    Comma,        // ,
    Dot,          // .
    SingleAnd,    // &
    InclusiveOr,  // |
    ExclusiveOr,  // ^
    Mod,          // %
    IDENTIFIER(String),
    IConstant(i64),
    FConstant(f64),
    StringLiteral(String, String),
    FuncName,    // __func__
    SIZEOF,      // sizeof
    PtrOp,       // ->
    IncOp,       // ++
    DecOp,       // --
    LeftOp,      // <<
    RightOp,     // >>
    LeOp,        // <=
    GeOp,        // >=
    EqOp,        // ==
    NeOp,        // !=
    AndOp,       // &&
    OrOp,        // ||
    MulAssign,   // *=
    DivAssign,   // /=
    ModAssign,   // %=
    AddAssign,   // +=
    SubAssign,   // -=
    LeftAssign,  // <<=
    RightAssign, // >>=
    AndAssign,   // &=
    XorAssign,   // ^=
    OrAssign,    // |=
    // TODO: this should be done when we found this is a typedef name,
    //       typedef LL int, then LL is typedef_name
    TypedefName,
    ELLIPSIS,                    // ..=
    EnumerationConstant(String), // TODO: add check
    TYPEDEF,
    EXTERN,
    STATIC,
    AUTO,
    REGISTER,
    INLINE,
    CONST,
    RESTRICT,
    VOLATILE,
    BOOL,
    CHAR,
    SHORT,
    INT,
    LONG,
    SIGNED,
    UNSIGNED,
    FLOAT,
    DOUBLE,
    VOID,
    COMPLEX,
    IMAGINARY,
    STRUCT,
    UNION,
    ENUM,
    CASE,
    DEFAULT,
    IF,
    ELSE,
    SWITCH,
    WHILE,
    DO,
    FOR,
    GOTO,
    CONTINUE,
    BREAK,
    RETURN,
    ALIGNAS,
    ALIGNOF,
    ATOMIC,
    GENERIC,
    NORETURN,
    StaticAssert,
    ThreadLocal,
}

use std::sync::atomic;

static LABEL_COUNTER: atomic::AtomicUsize = atomic::AtomicUsize::new(0);
fn gen_string_tag() -> String {
    let label_counter = LABEL_COUNTER.fetch_add(1, atomic::Ordering::SeqCst);
    let label = format!(".LSTR{}", label_counter);

    label
}

pub fn lex(input: &str) -> Result<Vec<TokType>, String> {
    let mut result = Vec::new();

    let mut it = input.chars().peekable();

    while let Some(&c) = it.peek() {
        match c {
            '"' => {
                it.next();
                let mut s = "".to_string();
                loop {
                    let &c = it.peek().unwrap();
                    if c == '"' {
                        break;
                    }
                    s.push(c);
                    it.next();
                }
                result.push(TokType::StringLiteral(s, gen_string_tag()));
                it.next();
            }
            '\'' => {
                // try parse a char
                it.next(); // skip '
                let &c = it.peek().unwrap();
                if c == '\'' {
                    return Err(format!("Error: empty character constant"));
                }
                if c == '\\' {
                    it.next();
                    let &c = it.peek().unwrap();
                    match c {
                        'a' => {
                            result.push(TokType::IConstant(0x07));
                        } // Alert (Beep, Bell) (added in C89)
                        'b' => {
                            result.push(TokType::IConstant(0x08));
                        } // Backspace
                        'e' => {
                            result.push(TokType::IConstant(0x1B));
                        } // escape character
                        'f' => {
                            result.push(TokType::IConstant(0x0C));
                        } // Formfeed Page Break
                        'n' => {
                            result.push(TokType::IConstant(0x0A));
                        } // Newline (Line Feed)
                        'r' => {
                            result.push(TokType::IConstant(0x0D));
                        } // Carriage Return
                        't' => {
                            result.push(TokType::IConstant(0x09));
                        } // Horizontal Tab
                        'v' => {
                            result.push(TokType::IConstant(0x0B));
                        } // Vertical Tab
                        '\\' => {
                            result.push(TokType::IConstant(0x5C));
                        } // Backslash
                        '\'' => {
                            result.push(TokType::IConstant(0x27));
                        } // Apostrophe or single quotation mark
                        '\"' => {
                            result.push(TokType::IConstant(0x22));
                        } // Double quotation mark
                        '?' => {
                            result.push(TokType::IConstant(0x3F));
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
                    result.push(TokType::IConstant(c as i64));
                    it.next(); // skip char
                    it.next(); // skip '
                }
            }
            '0'..='9' => {
                it.next();
                let mut number = c
                    .to_string()
                    .parse::<i64>()
                    .expect("The caller should have passed a digit.");

                while let Some(Ok(digit)) = it.peek().map(|c| c.to_string().parse::<i64>()) {
                    number = number * 10 + digit;
                    it.next();
                }
                result.push(TokType::IConstant(number));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                it.next();
                let mut s = String::new();
                s.push(c);
                while let Some(&tmp) = it.peek() {
                    match tmp {
                        'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                            s.push(tmp);
                            it.next();
                        }
                        _ => {
                            break;
                        }
                    }
                }
                match s.as_ref() {
                    "auto" => result.push(TokType::AUTO),
                    "break" => result.push(TokType::BREAK),
                    "case" => result.push(TokType::CASE),
                    "char" => result.push(TokType::CHAR),
                    "const" => result.push(TokType::CONST),
                    "continue" => result.push(TokType::CONTINUE),
                    "default" => result.push(TokType::DEFAULT),
                    "do" => result.push(TokType::DO),
                    "double" => result.push(TokType::DOUBLE),
                    "else" => result.push(TokType::ELSE),
                    "enum" => result.push(TokType::ENUM),
                    "extern" => result.push(TokType::EXTERN),
                    "float" => result.push(TokType::FLOAT),
                    "for" => result.push(TokType::FOR),
                    "goto" => result.push(TokType::GOTO),
                    "if" => result.push(TokType::IF),
                    "inline" => result.push(TokType::INLINE),
                    "int" => result.push(TokType::INT),
                    "long" => result.push(TokType::LONG),
                    "register" => result.push(TokType::REGISTER),
                    "restrict" => result.push(TokType::RESTRICT),
                    "return" => result.push(TokType::RETURN),
                    "short" => result.push(TokType::SHORT),
                    "signed" => result.push(TokType::SIGNED),
                    "sizeof" => result.push(TokType::SIZEOF),
                    "static" => result.push(TokType::STATIC),
                    "struct" => result.push(TokType::STRUCT),
                    "switch" => result.push(TokType::SWITCH),
                    "typedef" => result.push(TokType::TYPEDEF),
                    "union" => result.push(TokType::UNION),
                    "unsigned" => result.push(TokType::UNSIGNED),
                    "void" => result.push(TokType::VOID),
                    "volatile" => result.push(TokType::VOLATILE),
                    "while" => result.push(TokType::WHILE),
                    "_Alignas" => result.push(TokType::ALIGNAS),
                    "_Alignof" => result.push(TokType::ALIGNOF),
                    "_Atomic" => result.push(TokType::ATOMIC),
                    "_Bool" => result.push(TokType::BOOL),
                    "_Complex" => result.push(TokType::COMPLEX),
                    "_Generic" => result.push(TokType::GENERIC),
                    "_Imaginary" => result.push(TokType::IMAGINARY),
                    "_Noreturn" => result.push(TokType::NORETURN),
                    "_Static_assert" => result.push(TokType::StaticAssert),
                    "_Thread_local" => result.push(TokType::ThreadLocal),
                    "__func__" => result.push(TokType::FuncName),
                    _ => result.push(TokType::IDENTIFIER(s)),
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
                            result.push(TokType::EqOp);
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
                            result.push(TokType::LeOp);
                            it.next();
                        }
                        '<' => {
                            it.next();
                            match it.peek() {
                                Some(tmp) => match tmp {
                                    '=' => {
                                        it.next();
                                        result.push(TokType::LeftAssign); // <<=
                                        it.next();
                                    }
                                    _ => {
                                        result.push(TokType::LeftOp);
                                        it.next();
                                    }
                                },
                                _ => {
                                    result.push(TokType::LeftOp);
                                }
                            }
                        }
                        _ => {
                            result.push(TokType::Lt);
                        }
                    },
                    _ => {
                        result.push(TokType::Lt);
                    }
                }
            }
            '>' => {
                it.next();
                match it.peek() {
                    Some(tmp) => match tmp {
                        '=' => {
                            result.push(TokType::GeOp);
                            it.next();
                        }
                        '>' => {
                            it.next();
                            match it.peek() {
                                Some(tmp) => match tmp {
                                    '=' => {
                                        result.push(TokType::RightAssign);
                                        it.next();
                                    }
                                    _ => {
                                        result.push(TokType::RightOp);
                                    }
                                },
                                _ => {
                                    result.push(TokType::RightOp);
                                }
                            }
                        }
                        _ => {
                            result.push(TokType::Gt);
                        }
                    },
                    _ => {
                        result.push(TokType::Gt);
                    }
                }
            }
            '-' => {
                it.next();
                match it.peek() {
                    Some(tmp) => match tmp {
                        '-' => {
                            result.push(TokType::DecOp);
                            it.next();
                        }
                        '=' => {
                            result.push(TokType::SubAssign);
                            it.next();
                        }
                        '>' => {
                            result.push(TokType::PtrOp);
                            it.next();
                        }
                        _ => {
                            result.push(TokType::Minus);
                        }
                    },
                    _ => {
                        result.push(TokType::Minus);
                    }
                }
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
                            result.push(TokType::NeOp);
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
                it.next();
                match it.peek().unwrap() {
                    '+' => {
                        result.push(TokType::IncOp);
                        it.next();
                    }
                    '=' => {
                        result.push(TokType::AddAssign);
                        it.next();
                    }
                    _ => {
                        result.push(TokType::Plus);
                    }
                }
            }
            '*' => {
                it.next();
                match it.peek() {
                    Some(tmp) => match tmp {
                        '=' => {
                            result.push(TokType::MulAssign);
                            it.next();
                        }
                        _ => {
                            result.push(TokType::Multi);
                        }
                    },
                    _ => {
                        result.push(TokType::Multi);
                    }
                }
            }
            '%' => {
                it.next();
                match it.peek() {
                    Some(tmp) => match tmp {
                        '=' => {
                            result.push(TokType::ModAssign);
                            it.next();
                        }
                        _ => {
                            result.push(TokType::Mod);
                        }
                    },
                    _ => {
                        result.push(TokType::Mod);
                    }
                }
            }
            '/' => {
                it.next();
                match it.peek() {
                    Some(tmp) => match tmp {
                        '=' => {
                            result.push(TokType::DivAssign);
                            it.next();
                        }
                        _ => {
                            result.push(TokType::Splash);
                        }
                    },
                    _ => {
                        result.push(TokType::Splash);
                    }
                }
            }
            '&' => {
                it.next();
                match it.peek() {
                    Some(tmp) => match tmp {
                        '&' => {
                            result.push(TokType::AndOp);
                            it.next();
                        }
                        '=' => {
                            result.push(TokType::AndAssign);
                            it.next();
                        }
                        _ => {
                            // & operator to get the address of a variable
                            result.push(TokType::SingleAnd);
                        }
                    },
                    _ => {
                        result.push(TokType::SingleAnd);
                    }
                }
            }
            '|' => {
                it.next();
                match it.peek() {
                    Some(tmp) => match tmp {
                        '|' => {
                            result.push(TokType::OrOp);
                            it.next();
                        }
                        '=' => {
                            result.push(TokType::OrAssign);
                            it.next();
                        }
                        _ => {
                            // now don't support bitwise or, so just return Err
                            result.push(TokType::InclusiveOr);
                        }
                    },
                    _ => {
                        result.push(TokType::InclusiveOr);
                    }
                }
            }
            '^' => {
                it.next();
                match it.peek() {
                    Some(tmp) => match tmp {
                        '=' => {
                            result.push(TokType::XorAssign);
                            it.next();
                        }
                        _ => {
                            result.push(TokType::ExclusiveOr);
                        }
                    },
                    _ => {
                        result.push(TokType::ExclusiveOr);
                    }
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
            ' ' | '\n' | '\t' | '\r' | '#' => {
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
