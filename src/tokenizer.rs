use std::collections::VecDeque;

use crate::json_util;

// Todo: remove PartialEq and Eq to add Float
#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Null,
    Int(i64),
    // Float(f64),
    String(String),
    Boolean(bool),
    Colon,
    Comma,
    LeftSquareBrancket,
    RightSquareBrancket,
    LeftCurlyBranckt,
    RightCurlyBranckt,
    Eof,
}

pub struct Tokenizer {
    input: VecDeque<char>,
}

impl Iterator for Tokenizer {
    type Item = Result<Token, String>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(Token::Eof) => None,
            result => Some(result),
        }
    }
}

impl Tokenizer {
    pub fn new<S: Into<String>>(input: S) -> Self {
        let input = input.into().chars().collect::<VecDeque<char>>();
        Self { input }
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespaces();

        match self.front() {
            Some(c) if c.is_ascii_digit() => self.tokenize_number(),
            Some('-') => self.tokenize_number(),
            Some('{') => {
                self.pop();
                Ok(Token::LeftCurlyBranckt)
            }
            Some('}') => {
                self.pop();
                Ok(Token::RightCurlyBranckt)
            }
            Some('[') => {
                self.pop();
                Ok(Token::LeftSquareBrancket)
            }
            Some(']') => {
                self.pop();
                Ok(Token::RightSquareBrancket)
            }
            Some(':') => {
                self.pop();
                Ok(Token::Colon)
            }
            Some(',') => {
                self.pop();
                Ok(Token::Comma)
            }
            Some('\"') => self.tokenize_string(),
            Some('t') => self.tokenize_true(),
            Some('f') => self.tokenize_false(),
            Some('n') => self.tokenize_null(),
            None => Ok(Token::Eof),
            Some(c) => Err(format!(
                "The tokenizer found an unexpected character \'{:}\'.",
                c
            )),
        }
    }

    fn skip_whitespaces(&mut self) {
        loop {
            match self.front() {
                Some(&c) if json_util::is_whitespace(c) => self.pop(),
                _ => break,
            };
        }
    }

    fn tokenize_string(&mut self) -> Result<Token, String> {
        let mut ident = String::new();

        self.consume('\"')?;
        loop {
            match self.front() {
                Some('\\') => {
                    let escaped = self.pop_escape().ok_or(r#"The next of \ must be a escaped character"#)?;
                    ident.push(escaped);
                },
                Some('\"') => { self.pop(); break; },
                Some(&c) if json_util::is_unescaped(c) => {
                    self.pop();
                    ident.push(c);
                },
                None => return Err("The tokenizer reached EOF before finding \" which represents the end of a string".to_string()),
                _ => return Err("The tokenizer found a unexpected character while tokenizing string.".to_string()),
            }
        }

        Ok(Token::String(ident))
    }

    //Todo: fix to accpet float values
    fn tokenize_number(&mut self) -> Result<Token, String> {
        match self.front() {
            //Some('0') => Err("The head of number must not be zero"),
            Some('-') => {
                self.pop();
                let num = self.read_digits();
                Ok(Token::Int(-num))
            }
            _ => {
                let num = self.read_digits();
                Ok(Token::Int(num))
            }
        }
    }

    fn tokenize_true(&mut self) -> Result<Token, String> {
        self.consume('t')?;
        self.consume('r')?;
        self.consume('u')?;
        self.consume('e')?;

        Ok(Token::Boolean(true))
    }

    fn tokenize_false(&mut self) -> Result<Token, String> {
        self.consume('f')?;
        self.consume('a')?;
        self.consume('l')?;
        self.consume('s')?;
        self.consume('e')?;

        Ok(Token::Boolean(false))
    }

    fn tokenize_null(&mut self) -> Result<Token, String> {
        self.consume('n')?;
        self.consume('u')?;
        self.consume('l')?;
        self.consume('l')?;

        Ok(Token::Null)
    }

    // Todo: make the return type Result<i64, String>.
    fn read_digits(&mut self) -> i64 {
        let mut digits = String::new();

        while let Some(c) = self.pop_digit() {
            digits.push(c)
        }

        digits.parse().expect("digits must represent number.")
    }

    fn consume(&mut self, c: char) -> Result<char, String> {
        match self.pop() {
            Some(top) if top == c => Ok(top),
            Some(top) => Err(format!(
                "The tokenizer expected {:}, but found {:}.",
                c, top
            )),
            _ => Err(format!("The tokenizer expected {:}, but reached EOF.", c)),
        }
    }

    fn front(&self) -> Option<&char> {
        self.input.front()
    }

    fn pop(&mut self) -> Option<char> {
        self.input.pop_front()
    }

    fn pop_digit(&mut self) -> Option<char> {
        match self.front() {
            Some(c) if c.is_ascii_digit() => self.pop(),
            _ => None,
        }
    }

    // Todo: fix to remove the call of is_escape_target.
    fn pop_escape(&mut self) -> Option<char> {
        match self.front() {
            Some(&c) if json_util::is_escape_target(c) => {
                self.pop();
                json_util::escape(c)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Todo: define a function or macro to determine whether two tokens are same or not.

    #[test]
    #[rustfmt::skip]
    fn tokenize_empty() {
        let mut tokenizer = Tokenizer::new("");
        assert_eq!(tokenizer.next_token(), Ok(Token::Eof));
    }

    #[test]
    #[rustfmt::skip]
    fn tokenize_zero() {
        let mut tokenizer = Tokenizer::new("0");
        assert_eq!(tokenizer.next_token(), Ok(Token::Int(0)));
        assert_eq!(tokenizer.next_token(), Ok(Token::Eof));
    }

    #[test]
    #[rustfmt::skip]
    fn tokenize_int() {
        let mut tokenizer = Tokenizer::new("123");
        assert_eq!(tokenizer.next_token(), Ok(Token::Int(123)));
        assert_eq!(tokenizer.next_token(), Ok(Token::Eof));
    }

    #[test]
    #[rustfmt::skip]
    fn tokenize_minus_int() {
        let mut tokenizer = Tokenizer::new("-123");
        assert_eq!(tokenizer.next_token(), Ok(Token::Int(-123)));
        assert_eq!(tokenizer.next_token(), Ok(Token::Eof));
    }

    #[test]
    #[rustfmt::skip]
    fn tokenize_string() {
        let mut tokenizer = Tokenizer::new(r#""apple""#);
        assert_eq!(tokenizer.next_token(), Ok(Token::String("apple".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Eof));
    }

    #[test]
    #[rustfmt::skip]
    fn tokenize_true() {
        let mut tokenizer = Tokenizer::new(r#"true"#);
        assert_eq!(tokenizer.next_token(), Ok(Token::Boolean(true)));
        assert_eq!(tokenizer.next_token(), Ok(Token::Eof));
    }

    #[test]
    #[rustfmt::skip]
    fn tokenize_false() {
        let mut tokenizer = Tokenizer::new(r#"false"#);
        assert_eq!(tokenizer.next_token(), Ok(Token::Boolean(false)));
        assert_eq!(tokenizer.next_token(), Ok(Token::Eof));
    }

    #[test]
    #[rustfmt::skip]
    fn tokenize_null() {
        let mut tokenizer = Tokenizer::new(r#"null"#);
        assert_eq!(tokenizer.next_token(), Ok(Token::Null));
        assert_eq!(tokenizer.next_token(), Ok(Token::Eof));
    }

    #[test]
    #[rustfmt::skip]
    fn tokenzie_object() {
        let input = r#"{ "elm1" : 123, "elm2" : 456 , "elm3" : "apple", "elm4": false }"#;
        let mut tokenizer = Tokenizer::new(input);
        assert_eq!(tokenizer.next_token(), Ok(Token::LeftCurlyBranckt));

        assert_eq!(tokenizer.next_token(), Ok(Token::String("elm1".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Colon));
        assert_eq!(tokenizer.next_token(), Ok(Token::Int(123)));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::String("elm2".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Colon));
        assert_eq!(tokenizer.next_token(), Ok(Token::Int(456)));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::String("elm3".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Colon));
        assert_eq!(tokenizer.next_token(), Ok(Token::String("apple".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::String("elm4".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Colon));
        assert_eq!(tokenizer.next_token(), Ok(Token::Boolean(false)));

        assert_eq!(tokenizer.next_token(), Ok(Token::RightCurlyBranckt));
        assert_eq!(tokenizer.next_token(), Ok(Token::Eof));
    }

    #[test]
    #[rustfmt::skip]
    fn tokenzie_object_no_whitespaces() {
        let input = r#"{"elm1":123,"elm2":456,"elm3":"apple","elm4":false}"#;
        let mut tokenizer = Tokenizer::new(input);
        assert_eq!(tokenizer.next_token(), Ok(Token::LeftCurlyBranckt));

        assert_eq!(tokenizer.next_token(), Ok(Token::String("elm1".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Colon));
        assert_eq!(tokenizer.next_token(), Ok(Token::Int(123)));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::String("elm2".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Colon));
        assert_eq!(tokenizer.next_token(), Ok(Token::Int(456)));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::String("elm3".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Colon));
        assert_eq!(tokenizer.next_token(), Ok(Token::String("apple".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::String("elm4".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Colon));
        assert_eq!(tokenizer.next_token(), Ok(Token::Boolean(false)));

        assert_eq!(tokenizer.next_token(), Ok(Token::RightCurlyBranckt));
        assert_eq!(tokenizer.next_token(), Ok(Token::Eof));
    }

    #[test]
    #[rustfmt::skip]
    fn tokenize_list() {
        let input = r#"[ 123, 456 , "apple", true ]"#;
        let mut tokenizer = Tokenizer::new(input);
        assert_eq!(tokenizer.next_token(), Ok(Token::LeftSquareBrancket));

        assert_eq!(tokenizer.next_token(), Ok(Token::Int(123)));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::Int(456)));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::String("apple".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::Boolean(true)));

        assert_eq!(tokenizer.next_token(), Ok(Token::RightSquareBrancket));
        assert_eq!(tokenizer.next_token(), Ok(Token::Eof));
    }

    #[test]
    #[rustfmt::skip]
    fn tokenize_list_no_whitespaces() {
        let input = r#"[123,456,"apple",true]"#;
        let mut tokenizer = Tokenizer::new(input);

        assert_eq!(tokenizer.next_token(), Ok(Token::LeftSquareBrancket));

        assert_eq!(tokenizer.next_token(), Ok(Token::Int(123)));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::Int(456)));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::String("apple".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::Boolean(true)));

        assert_eq!(tokenizer.next_token(), Ok(Token::RightSquareBrancket));
        assert_eq!(tokenizer.next_token(), Ok(Token::Eof));
    }

    #[test]
    #[rustfmt::skip]
    fn tokenize_large_input1() {
        let input = format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
            r#"{"#,
            r#"   "Image": {"#,
            r#"       "Width":  800,"#,
            r#"       "Height": 600,"#,
            r#"       "Title":  "View from 15th Floor","#,
            r#"       "Thumbnail": {"#,
            r#"           "Url":    "http://www.example.com/image/481989943","#,
            r#"           "Height": 125,"#,
            r#"           "Width":  100"#,
            r#"       },"#,
            r#"       "Animated" : false,"#,
            r#"       "IDs": [116, 943, 234, 38793]"#,
            r#"     }"#,
            r#"}"#,
        );

        let mut tokenizer = Tokenizer::new(input);
        assert_eq!(tokenizer.next_token(), Ok(Token::LeftCurlyBranckt));
        assert_eq!(tokenizer.next_token(), Ok(Token::String("Image".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Colon));
        assert_eq!(tokenizer.next_token(), Ok(Token::LeftCurlyBranckt));

        assert_eq!(tokenizer.next_token(), Ok(Token::String("Width".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Colon));
        assert_eq!(tokenizer.next_token(), Ok(Token::Int(800)));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::String("Height".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Colon));
        assert_eq!(tokenizer.next_token(), Ok(Token::Int(600)));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::String("Title".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Colon));
        assert_eq!(tokenizer.next_token(), Ok(Token::String("View from 15th Floor".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::String("Thumbnail".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Colon));
        assert_eq!(tokenizer.next_token(), Ok(Token::LeftCurlyBranckt));

        assert_eq!(tokenizer.next_token(), Ok(Token::String("Url".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Colon));
        assert_eq!(tokenizer.next_token(), Ok(Token::String("http://www.example.com/image/481989943".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::String("Height".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Colon));
        assert_eq!(tokenizer.next_token(), Ok(Token::Int(125)));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::String("Width".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Colon));
        assert_eq!(tokenizer.next_token(), Ok(Token::Int(100)));

        assert_eq!(tokenizer.next_token(), Ok(Token::RightCurlyBranckt));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::String("Animated".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Colon));
        assert_eq!(tokenizer.next_token(), Ok(Token::Boolean(false)));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::String("IDs".to_string())));
        assert_eq!(tokenizer.next_token(), Ok(Token::Colon));

        assert_eq!(tokenizer.next_token(), Ok(Token::LeftSquareBrancket));

        assert_eq!(tokenizer.next_token(), Ok(Token::Int(116)));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::Int(943)));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::Int(234)));
        assert_eq!(tokenizer.next_token(), Ok(Token::Comma));

        assert_eq!(tokenizer.next_token(), Ok(Token::Int(38793)));

        assert_eq!(tokenizer.next_token(), Ok(Token::RightSquareBrancket));

        assert_eq!(tokenizer.next_token(), Ok(Token::RightCurlyBranckt));
        assert_eq!(tokenizer.next_token(), Ok(Token::RightCurlyBranckt));
    }

    #[test]
    #[rustfmt::skip]
    fn tokenzie_with_iterator() {
        let input = r#"{ "elm1" : 123, "elm2" : 456 , "elm3" : "apple", "elm4": false }"#;
        let mut tokenizer = Tokenizer::new(input);
        assert_eq!(tokenizer.next(), Some(Ok(Token::LeftCurlyBranckt)));

        assert_eq!(tokenizer.next(), Some(Ok(Token::String("elm1".to_string()))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::Colon)));
        assert_eq!(tokenizer.next(), Some(Ok(Token::Int(123))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::Comma)));

        assert_eq!(tokenizer.next(), Some(Ok(Token::String("elm2".to_string()))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::Colon)));
        assert_eq!(tokenizer.next(), Some(Ok(Token::Int(456))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::Comma)));

        assert_eq!(tokenizer.next(), Some(Ok(Token::String("elm3".to_string()))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::Colon)));
        assert_eq!(tokenizer.next(), Some(Ok(Token::String("apple".to_string()))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::Comma)));

        assert_eq!(tokenizer.next(), Some(Ok(Token::String("elm4".to_string()))));
        assert_eq!(tokenizer.next(), Some(Ok(Token::Colon)));
        assert_eq!(tokenizer.next(), Some(Ok(Token::Boolean(false))));

        assert_eq!(tokenizer.next(), Some(Ok(Token::RightCurlyBranckt)));
        assert_eq!(tokenizer.next(), None);
    }
}
