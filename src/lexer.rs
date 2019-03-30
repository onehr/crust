use std::iter::Peekable;

pub mod lexer {

    #[derive(Eq, PartialEq, Clone, Debug)]
    pub enum TokenType {
        LBrace,    // {
        RBrace,    // }
        LParen,    // (
        RParen,    // )
        SEMICOLON, // ;
        INT,       // int
        RET,       // return
        IDENTIFIER(String),
        LITERAL(i64), // [0-9]+
    }

    pub fn lex(input: &String) -> Result<Vec<TokenType>, String> {
        let mut result = Vec::new();

        let mut it = input.chars().peekable();

        while let Some(&c) = it.peek() {
            match c {
                '0'...'9' => {
                    it.next();
                    let mut number = c.to_string().parse::<i64>()
                        .expect("The caller should have passed a digit.");
                    while let Some(Ok(digit)) = it.peek()
                        .map(|c| c.to_string().parse::<i64>()) {
                            number = number * 10 + digit;
                            it.next();
                        }
                    result.push(TokenType::LITERAL(number));
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
                            _ => {break;}
                        }
                    }
                    match s.as_ref() {
                        "int" => result.push(TokenType::INT),
                        "return" => result.push(TokenType::RET),
                        _ => result.push(TokenType::IDENTIFIER(s)),
                    }
                }
                '(' => {
                    result.push(TokenType::LParen);
                    it.next();
                }
                ')' => {
                    result.push(TokenType::RParen);
                    it.next();
                }
                '{' => {
                    result.push(TokenType::LBrace);
                    it.next();
                }
                '}' => {
                    result.push(TokenType::RBrace);
                    it.next();
                }
                ';' => {
                    result.push(TokenType::SEMICOLON);
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
