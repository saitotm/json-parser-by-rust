use std::collections::{HashMap, VecDeque};

use crate::tokenizer::Token;

#[derive(Debug, PartialEq, Eq)]
enum Node {
    Object(HashMap<String, Node>),
    Array(Vec<Node>),
    Boolean(bool),
    Int(i64),
    // Float(f64),
    JsonString(String),
}

struct Parser {
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
            Some(token) => Err(format!("Parse found an unexpected token {:#?} while parsing value.", token)),
            None => Err("Parse found an unexpected token while parsing value.".to_string()),
        }
    }

    fn consume(&mut self, token: Token) -> Result<(), String> {
        match self.pop() {
            Some(head) if head == token => Ok(()),
            Some(head) => Err(format!("Expected a token {:#?}, but found an unexpected token {:#?}", token, head)),
            None => Err(format!("Expected a token {:#?}", token)),
        }
    }

    fn assume(&mut self, token: Token) -> bool {
        match self.front() {
            Some(head) if head == &token => { self.pop(); true },
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
        let mut kvm = HashMap::new();
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

    // Todo: add colon
    fn array(&mut self) -> Result<Node, String> {
        let mut values = Vec::new();
        self.consume(Token::LeftSquareBrancket)?;

        loop {
            if self.assume(Token::RightSquareBrancket) {
                break;
            }

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

    fn string(&mut self) -> Result<Node, String> {
        match self.pop() {
            Some(Token::JsonString(value)) => Ok(Node::JsonString(value)),
            _ => Err("Parse found an unexpected token while parsing string.".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, VecDeque};

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
            HashMap::from([
                ("elm1".to_string(), Node::Int(123)), 
                ("elm2".to_string(), Node::Int(456)), 
                ("elm3".to_string(), Node::JsonString("apple".to_string())), 
                ("elm4".to_string(), Node::Boolean(false))
            ]));
        let node = Parser::new(tokens).parse();

        assert_eq!(node, Ok(expected));
    }
}
