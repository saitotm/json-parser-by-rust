use std::collections::VecDeque;

use indexmap::IndexMap;

use crate::tokenizer::Token;

#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    Null,
    Object(IndexMap<String, Node>),
    Array(Vec<Node>),
    Boolean(bool),
    Int(i64),
    // Float(f64),
    JsonString(String),
}

pub struct Parser {
    tokens: VecDeque<Token>,
}

impl Parser {
    pub fn new(tokens: VecDeque<Token>) -> Self {
        Self { tokens }
    }

    pub fn parse(&mut self) -> Result<Node, String> {
        self.json_text()
    }

    fn json_text(&mut self) -> Result<Node, String> {
        self.value()
    }

    // Todo: fix to accpet null
    fn value(&mut self) -> Result<Node, String> {
        match self.front() {
            Some(Token::LeftCurlyBranckt) => self.object(),
            Some(Token::LeftSquareBrancket) => self.array(),
            Some(Token::Int(_)) => self.int(),
            Some(Token::JsonString(_)) => self.string(),
            Some(Token::Boolean(_)) => self.boolean(),
            Some(Token::Null) => self.null(),
            Some(token) => Err(format!(
                "Parse found an unexpected token {:#?} while parsing value.",
                token
            )),
            None => Err("Parse found an unexpected token while parsing value.".to_string()),
        }
    }

    fn consume(&mut self, token: Token) -> Result<(), String> {
        match self.pop() {
            Some(head) if head == token => Ok(()),
            Some(head) => Err(format!(
                "Expected a token {:#?}, but found an unexpected token {:#?}",
                token, head
            )),
            None => Err(format!("Expected a token {:#?}", token)),
        }
    }

    fn assume(&mut self, token: Token) -> bool {
        match self.front() {
            Some(head) if head == &token => {
                self.pop();
                true
            }
            _ => false,
        }
    }

    fn front(&self) -> Option<&Token> {
        self.tokens.front()
    }

    fn pop(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }

    fn object(&mut self) -> Result<Node, String> {
        let mut kvm = IndexMap::new();
        self.consume(Token::LeftCurlyBranckt)?;

        if self.assume(Token::RightCurlyBranckt) {
            return Ok(Node::Object(kvm));
        }

        let (key, value) = self.member()?;
        kvm.insert(key, value);

        loop {
            if self.assume(Token::RightCurlyBranckt) {
                break;
            }

            self.consume(Token::Comma)?;
            let (key, value) = self.member()?;
            kvm.insert(key, value);
        }

        Ok(Node::Object(kvm))
    }

    fn member(&mut self) -> Result<(String, Node), String> {
        let key = match self.string()? {
            Node::JsonString(value) => value,
            _ => unreachable!(),
        };

        self.consume(Token::Colon)?;

        let value = self.value()?;

        Ok((key, value))
    }

    fn array(&mut self) -> Result<Node, String> {
        let mut values = Vec::new();
        self.consume(Token::LeftSquareBrancket)?;

        if self.assume(Token::RightSquareBrancket) {
            return Ok(Node::Array(values));
        }

        values.push(self.value()?);

        loop {
            if self.assume(Token::RightSquareBrancket) {
                break;
            }
            self.consume(Token::Comma)?;

            values.push(self.value()?);
        }

        Ok(Node::Array(values))
    }

    fn int(&mut self) -> Result<Node, String> {
        match self.pop() {
            Some(Token::Int(num)) => Ok(Node::Int(num)),
            _ => Err("Parse found an unexpected token while parsing int.".to_string()),
        }
    }

    fn boolean(&mut self) -> Result<Node, String> {
        match self.pop() {
            Some(Token::Boolean(v)) => Ok(Node::Boolean(v)),
            _ => Err("Parse found an unexpected token while parsing boolean.".to_string()),
        }
    }

    fn null(&mut self) -> Result<Node, String> {
        match self.pop() {
            Some(Token::Null) => Ok(Node::Null),
            _ => Err("Parse found an unexpected token while parsing null.".to_string()),
        }
    }

    fn string(&mut self) -> Result<Node, String> {
        match self.pop() {
            Some(Token::JsonString(value)) => Ok(Node::JsonString(value)),
            _ => Err("Parse found an unexpected token while parsing string.".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use indexmap::IndexMap;

    use crate::{
        parser::{Node, Parser},
        tokenizer::Token,
    };

    #[test]
    fn parse_int() {
        let mut tokens = VecDeque::new();
        tokens.push_back(Token::Int(123));
        tokens.push_back(Token::Eof);

        let expected = Node::Int(123);
        let node = Parser::new(tokens).parse();

        assert_eq!(node, Ok(expected));
    }

    #[test]
    fn parse_boolean() {
        let mut tokens = VecDeque::new();
        tokens.push_back(Token::Boolean(true));
        tokens.push_back(Token::Eof);

        let expected = Node::Boolean(true);
        let node = Parser::new(tokens).parse();

        assert_eq!(node, Ok(expected));
    }

    #[test]
    fn parse_null() {
        let mut tokens = VecDeque::new();
        tokens.push_back(Token::Null);
        tokens.push_back(Token::Eof);

        let expected = Node::Null;
        let node = Parser::new(tokens).parse();

        assert_eq!(node, Ok(expected));
    }

    #[test]
    fn parse_object() {
        let mut tokens = VecDeque::new();
        tokens.push_back(Token::LeftCurlyBranckt);

        tokens.push_back(Token::JsonString("elm1".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::Int(123));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::JsonString("elm2".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::Int(456));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::JsonString("elm3".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::JsonString("apple".to_string()));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::JsonString("elm4".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::Boolean(false));

        tokens.push_back(Token::RightCurlyBranckt);
        tokens.push_back(Token::Eof);

        #[rustfmt::skip]
        let expected = Node::Object(
            IndexMap::from([
                ("elm1".to_string(), Node::Int(123)), 
                ("elm2".to_string(), Node::Int(456)), 
                ("elm3".to_string(), Node::JsonString("apple".to_string())), 
                ("elm4".to_string(), Node::Boolean(false))
            ]));
        let node = Parser::new(tokens).parse();

        assert_eq!(node, Ok(expected));
    }

    #[test]
    fn parse_array() {
        let mut tokens = VecDeque::new();

        tokens.push_back(Token::LeftSquareBrancket);

        tokens.push_back(Token::Int(123));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::Int(456));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::JsonString("apple".to_string()));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::Boolean(true));

        tokens.push_back(Token::RightSquareBrancket);
        tokens.push_back(Token::Eof);

        #[rustfmt::skip]
        let expected = Node::Array(
            Vec::from([
                Node::Int(123),
                Node::Int(456),
                Node::JsonString("apple".to_string()),
                Node::Boolean(true)
            ]));
        let node = Parser::new(tokens).parse();

        assert_eq!(node, Ok(expected));
    }

    #[test]
    fn parse_large_json1() {
        let mut tokens = VecDeque::new();

        tokens.push_back(Token::LeftCurlyBranckt);
        tokens.push_back(Token::JsonString("Image".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::LeftCurlyBranckt);

        tokens.push_back(Token::JsonString("Width".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::Int(800));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::JsonString("Height".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::Int(600));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::JsonString("Title".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::JsonString("View from 15th Floor".to_string()));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::JsonString("Thumbnail".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::LeftCurlyBranckt);

        tokens.push_back(Token::JsonString("Url".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::JsonString(
            "http://www.example.com/image/481989943".to_string(),
        ));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::JsonString("Height".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::Int(125));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::JsonString("Width".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::Int(100));

        tokens.push_back(Token::RightCurlyBranckt);
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::JsonString("Animated".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::Boolean(false));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::JsonString("IDs".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::LeftSquareBrancket);

        tokens.push_back(Token::Int(116));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::Int(943));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::Int(234));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::Int(38793));

        tokens.push_back(Token::RightSquareBrancket);

        tokens.push_back(Token::RightCurlyBranckt);
        tokens.push_back(Token::RightCurlyBranckt);

        #[rustfmt::skip]
        let expected = Node::Object(
            IndexMap::from([
                ("Image".to_string(), Node::Object(
                        IndexMap::from([
                            ("Width".to_string(), Node::Int(800)),
                            ("Height".to_string(), Node::Int(600)),
                            ("Title".to_string(), Node::JsonString("View from 15th Floor".to_string())),
                            ("Thumbnail".to_string(), Node::Object(
                                    IndexMap::from([
                                        ("Url".to_string(), Node::JsonString("http://www.example.com/image/481989943".to_string())),
                                        ("Height".to_string(), Node::Int(125)),
                                        ("Width".to_string(), Node::Int(100)) 
                                    ]))
                            ),
                            ("Animated".to_string(), Node::Boolean(false)),
                            ("IDs".to_string(), Node::Array(Vec::from([
                                    Node::Int(116),
                                    Node::Int(943),
                                    Node::Int(234),
                                    Node::Int(38793) 
                            ])))
                        ])
                ))
            ]));
        let node = Parser::new(tokens).parse();

        assert_eq!(node, Ok(expected));
    }
}
