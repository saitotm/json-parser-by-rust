use indexmap::IndexMap;

use crate::parser::Node;

pub struct Generator {
    node: Node,
    indent_size: usize,
}

impl Generator {
    pub fn new(node: Node, indent_size: usize) -> Self {
        Self { node, indent_size }
    }

    pub fn generate(&self) -> String {
        self.generate_impl(&self.node, "")
    }

    fn inc_indent(&self, value: &str, indent_size: usize) -> String {
        format!("{}{}", " ".repeat(indent_size), value)
    }

    fn add_prefix(&self, value: String, prefix: &str) -> String {
        format!("{}{}", prefix, value)
    }

    fn generate_impl(&self, node: &Node, prefix: &str) -> String {
        match node {
            Node::Null => "null".to_string(),
            Node::Number(num) => num.to_string(),
            Node::String(value) => self.generate_string(value.to_string()),
            Node::Boolean(b) => b.to_string(),
            Node::Object(kvm) => self.generate_object(kvm, prefix),
            Node::Array(arr) => self.generate_array(arr, prefix),
        }
    }

    fn generate_string(&self, value: String) -> String {
        format!("\"{}\"", value)
    }

    fn generate_object_inner(&self, kvm: &IndexMap<String, Node>, prefix: &str) -> String {
        let new_prefix = self.inc_indent(prefix, self.indent_size);

        let mut inner = String::new();
        for (key, node) in kvm {
            let member = format!(
                "{}: {},\n",
                self.generate_string(key.to_string()),
                self.generate_impl(node, &new_prefix)
            );
            let member = self.add_prefix(member, &new_prefix);
            inner = format!("{}{}", inner, member);
        }

        // delete the end of comma and \n
        inner.pop();
        inner.pop();

        inner
    }

    fn generate_object(&self, kvm: &IndexMap<String, Node>, prefix: &str) -> String {
        format!(
            "{{\n{}\n{}}}",
            self.generate_object_inner(kvm, prefix),
            prefix
        )
    }

    fn generate_array(&self, arr: &[Node], prefix: &str) -> String {
        format!("[\n{}\n{}]", self.generate_array_inner(arr, prefix), prefix)
    }

    fn generate_array_inner(&self, arr: &[Node], prefix: &str) -> String {
        let new_prefix = self.inc_indent(prefix, self.indent_size);

        let mut inner = String::new();
        for node in arr {
            let elm = self.add_prefix(self.generate_impl(node, &new_prefix), &new_prefix);
            inner = format!("{}{},\n", inner, elm);
        }

        // delete the end of comma and \n
        inner.pop();
        inner.pop();

        inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_int() {
        let node = Node::Number("123".to_string());
        let gen = Generator::new(node, 4);

        assert_eq!(gen.generate(), "123");
    }

    #[test]
    fn generate_boolean() {
        let node = Node::Boolean(true);
        let gen = Generator::new(node, 4);

        assert_eq!(gen.generate(), "true");
    }

    #[test]
    fn generate_string() {
        let node = Node::String("apple".to_string());
        let gen = Generator::new(node, 4);

        assert_eq!(gen.generate(), "\"apple\"");
    }

    #[test]
    fn generate_null() {
        let node = Node::Null;
        let gen = Generator::new(node, 4);

        assert_eq!(gen.generate(), "null");
    }

    #[test]
    fn generate_object() {
        let node = Node::Object(IndexMap::from([
            ("elm1".to_string(), Node::Number("123".to_string())),
            ("elm2".to_string(), Node::Number("456".to_string())),
            ("elm3".to_string(), Node::String("apple".to_string())),
            ("elm4".to_string(), Node::Boolean(false)),
        ]));
        let gen = Generator::new(node, 4);

        #[rustfmt::skip]
        assert_eq!(
            gen.generate(),
            format!("{}\n{}\n{}\n{}\n{}\n{}",
                r#"{"#,
                r#"    "elm1": 123,"#,
                r#"    "elm2": 456,"#,
                r#"    "elm3": "apple","#,
                r#"    "elm4": false"#,
                r#"}"#
        ));
    }

    #[test]
    fn generate_array() {
        let node = Node::Array(Vec::from([
            Node::Number("123".to_string()),
            Node::Number("456".to_string()),
            Node::String("apple".to_string()),
            Node::Boolean(true),
        ]));
        let gen = Generator::new(node, 4);

        #[rustfmt::skip]
        assert_eq!(
            gen.generate(),
            format!("{}\n{}\n{}\n{}\n{}\n{}",
                r#"["#,
                r#"    123,"#,
                r#"    456,"#,
                r#"    "apple","#,
                r#"    true"#,
                r#"]"#
        ));
    }

    #[test]
    fn generate_large_json1() {
        #[rustfmt::skip]
        let node = Node::Object(
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
        let gen = Generator::new(node, 4);

        #[rustfmt::skip]
        assert_eq!(
            gen.generate(),
            format!("{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
                r#"{"#,
                r#"    "Image": {"#,
                r#"        "Width": 800,"#,
                r#"        "Height": 600,"#,
                r#"        "Title": "View from 15th Floor","#,
                r#"        "Thumbnail": {"#,
                r#"            "Url": "http://www.example.com/image/481989943","#,
                r#"            "Height": 125,"#,
                r#"            "Width": 100"#,
                r#"        },"#,
                r#"        "Animated": false,"#,
                r#"        "IDs": ["#,
                r#"            116,"#, 
                r#"            943,"#, 
                r#"            234,"#, 
                r#"            38793"#,
                r#"        ]"#,
                r#"    }"#,
                r#"}"#,
        ));
    }
}
