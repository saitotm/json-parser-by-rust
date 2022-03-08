use indexmap::IndexMap;

use crate::parser::Node;

pub struct Generator {
    node: Node,
}

fn generate_impl(node: &Node) -> String {
    match node {
        Node::Int(num) => num.to_string(),
        Node::JsonString(value) => generate_string(value),
        Node::Boolean(b) => b.to_string(),
        Node::Object(kvm) => generate_object(kvm),
        Node::Array(arr) => generate_array(arr),
        _ => unimplemented!(),
    }
}

fn generate_string(value: &str) -> String {
    format!("\"{}\"", value)
}

fn generate_object_inner(kvm: &IndexMap<String, Node>) -> String {
    let mut inner = String::new();
    for (key, node) in kvm {
        inner = format!(
            "{}    {}: {},\n",
            inner,
            generate_string(key),
            generate_impl(node)
        );
    }

    // delete the end of comma and \n
    inner.pop();
    inner.pop();

    inner
}

fn generate_object(kvm: &IndexMap<String, Node>) -> String {
    format!("{{\n{}\n}}", generate_object_inner(kvm))
}

fn generate_array(arr: &[Node]) -> String {
    format!("[\n{}\n]", generate_array_inner(arr))
}

fn generate_array_inner(arr: &[Node]) -> String {
    let mut inner = String::new();
    for node in arr {
        inner = format!("{}    {},\n", inner, generate_impl(node));
    }

    // delete the end of comma and \n
    inner.pop();
    inner.pop();

    inner
}

impl Generator {
    pub fn new(node: Node) -> Self {
        Self { node }
    }

    pub fn generate(&self) -> String {
        generate_impl(&self.node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_int() {
        let node = Node::Int(123);
        let gen = Generator::new(node);

        assert_eq!(gen.generate(), "123");
    }

    #[test]
    fn generate_boolean() {
        let node = Node::Boolean(true);
        let gen = Generator::new(node);

        assert_eq!(gen.generate(), "true");
    }

    #[test]
    fn generate_string() {
        let node = Node::JsonString("apple".to_string());
        let gen = Generator::new(node);

        assert_eq!(gen.generate(), "\"apple\"");
    }

    #[test]
    fn generate_object() {
        let node = Node::Object(IndexMap::from([
            ("elm1".to_string(), Node::Int(123)),
            ("elm2".to_string(), Node::Int(456)),
            ("elm3".to_string(), Node::JsonString("apple".to_string())),
            ("elm4".to_string(), Node::Boolean(false)),
        ]));
        let gen = Generator::new(node);

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
            Node::Int(123),
            Node::Int(456),
            Node::JsonString("apple".to_string()),
            Node::Boolean(true),
        ]));
        let gen = Generator::new(node);

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
    #[ignore]
    fn generate_large_json1() {
        #[rustfmt::skip]
        let node = Node::Object(
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
        let gen = Generator::new(node);

        #[rustfmt::skip]
        assert_eq!(
            gen.generate(),
            format!("{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
                r#"{"#,
                r#"  "Image": {"#,
                r#"      "Width":  800,"#,
                r#"      "Height": 600,"#,
                r#"      "Title":  "View from 15th Floor","#,
                r#"      "Thumbnail": {"#,
                r#"          "Url":    "http://www.example.com/image/481989943","#,
                r#"          "Height": 125,"#,
                r#"          "Width":  100"#,
                r#"      },"#,
                r#"      "Animated" : false,"#,
                r#"      "IDs": [116, 943, 234, 38793]"#,
                r#"    }"#,
                r#"}"#,
        ));
    }
}
