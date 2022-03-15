use std::collections::VecDeque;

use indexmap::IndexMap;

use crate::tokenizer::Token;

#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    Null,
    Object(IndexMap<String, Node>),
    Array(Vec<Node>),
    Boolean(bool),
    Number(String),
    String(String),
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

    fn value(&mut self) -> Result<Node, String> {
        match self.front() {
            Some(Token::LeftCurlyBranckt) => self.object(),
            Some(Token::LeftSquareBrancket) => self.array(),
            Some(Token::Number(_)) => self.int(),
            Some(Token::String(_)) => self.string(),
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
            Node::String(value) => value,
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
            Some(Token::Number(num)) => Ok(Node::Number(num)),
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
            Some(Token::String(value)) => Ok(Node::String(value)),
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
        tokens.push_back(Token::Number("123".to_string()));
        tokens.push_back(Token::Eof);

        let expected = Node::Number("123".to_string());
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

        tokens.push_back(Token::String("elm1".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::Number("123".to_string()));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::String("elm2".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::Number("456".to_string()));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::String("elm3".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::String("apple".to_string()));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::String("elm4".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::Boolean(false));

        tokens.push_back(Token::RightCurlyBranckt);
        tokens.push_back(Token::Eof);

        #[rustfmt::skip]
        let expected = Node::Object(
            IndexMap::from([
                ("elm1".to_string(), Node::Number("123".to_string())), 
                ("elm2".to_string(), Node::Number("456".to_string())), 
                ("elm3".to_string(), Node::String("apple".to_string())), 
                ("elm4".to_string(), Node::Boolean(false))
            ]));
        let node = Parser::new(tokens).parse();

        assert_eq!(node, Ok(expected));
    }

    #[test]
    fn parse_array() {
        let mut tokens = VecDeque::new();

        tokens.push_back(Token::LeftSquareBrancket);

        tokens.push_back(Token::Number("123".to_string()));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::Number("456".to_string()));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::String("apple".to_string()));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::Boolean(true));

        tokens.push_back(Token::RightSquareBrancket);
        tokens.push_back(Token::Eof);

        #[rustfmt::skip]
        let expected = Node::Array(
            Vec::from([
                Node::Number("123".to_string()),
                Node::Number("456".to_string()),
                Node::String("apple".to_string()),
                Node::Boolean(true)
            ]));
        let node = Parser::new(tokens).parse();

        assert_eq!(node, Ok(expected));
    }

    #[test]
    fn parse_large_json1() {
        let mut tokens = VecDeque::new();

        tokens.push_back(Token::LeftCurlyBranckt);
        tokens.push_back(Token::String("Image".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::LeftCurlyBranckt);

        tokens.push_back(Token::String("Width".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::Number("800".to_string()));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::String("Height".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::Number("600".to_string()));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::String("Title".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::String("View from 15th Floor".to_string()));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::String("Thumbnail".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::LeftCurlyBranckt);

        tokens.push_back(Token::String("Url".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::String(
            "http://www.example.com/image/481989943".to_string(),
        ));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::String("Height".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::Number("125".to_string()));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::String("Width".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::Number("100".to_string()));

        tokens.push_back(Token::RightCurlyBranckt);
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::String("Animated".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::Boolean(false));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::String("IDs".to_string()));
        tokens.push_back(Token::Colon);
        tokens.push_back(Token::LeftSquareBrancket);

        tokens.push_back(Token::Number("116".to_string()));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::Number("943".to_string()));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::Number("234".to_string()));
        tokens.push_back(Token::Comma);

        tokens.push_back(Token::Number("38793".to_string()));

        tokens.push_back(Token::RightSquareBrancket);

        tokens.push_back(Token::RightCurlyBranckt);
        tokens.push_back(Token::RightCurlyBranckt);

        #[rustfmt::skip]
        let expected = Node::Object(
            IndexMap::from([
                ("Image".to_string(), Node::Object(
                        IndexMap::from([
                            ("Width".to_string(), Node::Number("800".to_string())),
                            ("Height".to_string(), Node::Number("600".to_string())),
                            ("Title".to_string(), Node::String("View from 15th Floor".to_string())),
                            ("Thumbnail".to_string(), Node::Object(
                                    IndexMap::from([
                                        ("Url".to_string(), Node::String("http://www.example.com/image/481989943".to_string())),
                                        ("Height".to_string(), Node::Number("125".to_string())),
                                        ("Width".to_string(), Node::Number("100".to_string())) 
                                    ]))
                            ),
                            ("Animated".to_string(), Node::Boolean(false)),
                            ("IDs".to_string(), Node::Array(Vec::from([
                                    Node::Number("116".to_string()),
                                    Node::Number("943".to_string()),
                                    Node::Number("234".to_string()),
                                    Node::Number("38793".to_string()) 
                            ])))
                        ])
                ))
            ]));
        let node = Parser::new(tokens).parse();

        assert_eq!(node, Ok(expected));
    }
}
