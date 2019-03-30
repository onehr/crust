use std::iter::Peekable;

pub mod lexer {

    #[derive(Eq, PartialEq, Clone, Debug)]
    pub enum KwdType {
        INT,   // int
        VOID,  // void
        RET,   // return
        IF,    // if
        ELSE,  // else
        WHILE, // while
        FOR,   // for
    }

    #[derive(Eq, PartialEq, Clone, Debug)]
    pub enum TokType {
        Kwd(KwdType),
        LBrace,             // {
        RBrace,             // }
        LParen,             // (
        RParen,             // )
        SEMICOLON,          // ;
        EQ,                 // =
        LT,                 // <
        GT,                 // >
        PLUS,               // +
        MINUS,              // -
        LITERAL(i64),       // [0-9]+
        IDENTIFIER(String), // identifier
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
                    result.push(TokType::LITERAL(number));
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
                        "int" => result.push(TokType::Kwd(KwdType::INT)),
                        "return" => result.push(TokType::Kwd(KwdType::RET)),
                        "void" => result.push(TokType::Kwd(KwdType::VOID)),
                        "if" => result.push(TokType::Kwd(KwdType::IF)),
                        "else" => result.push(TokType::Kwd(KwdType::ELSE)),
                        "while" => result.push(TokType::Kwd(KwdType::WHILE)),
                        "for" => result.push(TokType::Kwd(KwdType::FOR)),
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
                ';' => {
                    result.push(TokType::SEMICOLON);
                    it.next();
                }
                '=' => {
                    result.push(TokType::EQ);
                    it.next();
                }
                '<' => {
                    result.push(TokType::LT);
                    it.next();
                }
                '>' => {
                    result.push(TokType::GT);
                    it.next();
                }
                '+' => {
                    result.push(TokType::PLUS);
                    it.next();
                }
                '-' => {
                    result.push(TokType::MINUS);
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
}
