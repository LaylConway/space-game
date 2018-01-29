//! Templates parsed in from markup

mod component;

mod parser {
    #[derive(Parser)]
    #[grammar = "grammar/template.pest"]
    pub struct TemplateParser;
}

pub use self::component::{ComponentTemplate};

#[cfg(test)]
mod test {
    use template::{ComponentTemplate};
    use {Value};

    #[test]
    fn it_parses_single_root() {
        let result = ComponentTemplate::from_str("root\n");

        println!("Result: {:?}", result);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().class, "root");
    }

    #[test]
    fn it_parses_root_with_child() {
        let result = ComponentTemplate::from_str("root\n    child\n");

        println!("Result: {:?}", result);
        assert!(result.is_ok());
        let component = result.unwrap();
        assert_eq!(component.class, "root");
        assert_eq!(component.children.len(), 1, "Incorrect children length on root");
        assert_eq!(component.children[0].class, "child");
    }

    #[test]
    fn it_parses_root_with_nested_children() {
        let result = ComponentTemplate::from_str("root\n    child\n        nested_child\n");

        println!("Result: {:?}", result);
        assert!(result.is_ok());
        let component = result.unwrap();
        assert_eq!(component.class, "root");
        assert_eq!(component.children.len(), 1, "Incorrect children length on root");
        assert_eq!(component.children[0].class, "child");
        assert_eq!(component.children[0].children.len(), 1, "Incorrect children length on child");
        assert_eq!(component.children[0].children[0].class, "nested_child");
    }

    #[test]
    fn it_parses_root_with_two_children() {
        let result = ComponentTemplate::from_str("root\n    child1\n    child2\n");

        println!("Result: {:?}", result);
        assert!(result.is_ok());
        let component = result.unwrap();
        assert_eq!(component.class, "root");
        assert_eq!(component.children.len(), 2, "Incorrect children length on root");
        assert_eq!(component.children[0].class, "child1");
        assert_eq!(component.children[1].class, "child2");
    }

    #[test]
    fn it_parses_varied_children_depth() {
        let result = ComponentTemplate::from_str("root\n    child1\n        nested_child\n    child2\n");

        println!("Result: {:?}", result);
        assert!(result.is_ok());
        let component = result.unwrap();
        assert_eq!(component.class, "root");
        assert_eq!(component.children.len(), 2, "Incorrect children length on root");
        assert_eq!(component.children[0].class, "child1");
        assert_eq!(component.children[1].class, "child2");
        assert_eq!(component.children[0].children.len(), 1, "Incorrect children length on child1");
        assert_eq!(component.children[0].children[0].class, "nested_child");
    }

    #[test]
    fn it_parses_root_attributes() {
        let result = ComponentTemplate::from_str("root { key: \"value\" }\n");

        println!("Result: {:?}", result);
        assert!(result.is_ok());
        let component = result.unwrap();
        assert_eq!(component.class, "root");
        assert_eq!(component.attributes.len(), 1);
        assert_eq!(component.attributes.get("key"), Some(&Value::String("value".into())));
    }

    #[test]
    fn it_parses_newlines_in_attributes_while_parsing_children() {
        let result = ComponentTemplate::from_str(
r#"root {
    key: "value",
    key2: "value2",
}
    child
"#
        );

        println!("Result: {:?}", result);
        assert!(result.is_ok());
        let component = result.unwrap();
        assert_eq!(component.class, "root");
        assert_eq!(component.children.len(), 1, "Incorrect children length on root");
        assert_eq!(component.children[0].class, "child");
    }

    #[test]
    fn it_parses_number_attributes() {
        let result = ComponentTemplate::from_str("root { key1: 5, key2: 2.5, key3: 69% }\n");

        println!("Result: {:?}", result);
        assert!(result.is_ok());
        let component = result.unwrap();
        assert_eq!(component.class, "root");
        assert_eq!(component.attributes.len(), 3);
        assert_eq!(component.attributes.get("key1"), Some(&Value::Integer(5)));
        assert_eq!(component.attributes.get("key2"), Some(&Value::Float(2.5)));
        assert_eq!(component.attributes.get("key3"), Some(&Value::Percentage(69)));
    }

    #[test]
    fn it_parses_tuple_attributes() {
        let result = ComponentTemplate::from_str("root { key: (50, \"text\") }\n");

        println!("Result: {:?}", result);
        assert!(result.is_ok());
        let component = result.unwrap();
        assert_eq!(component.class, "root");
        assert_eq!(component.attributes.len(), 1);
        assert_eq!(
            component.attributes.get("key"),
            Some(&Value::Tuple(vec!(Value::Integer(50), Value::String("text".into()))))
        );
    }

    #[test]
    fn it_fails_two_roots() {
        let result = ComponentTemplate::from_str("root\nroot2\n");

        println!("Result: {:?}", result);
        assert!(result.is_err());
    }

    #[test]
    fn it_fails_excessive_indentation() {
        let result = ComponentTemplate::from_str("root\n        excessive_child1\n");

        println!("Result: {:?}", result);
        assert!(result.is_err());
    }

    #[test]
    fn it_fails_non_4_indentation() {
        let result1 = ComponentTemplate::from_str("root\n  bad_child\n");
        let result2 = ComponentTemplate::from_str("root\n     bad_child\n");

        println!("Result1: {:?}", result1);
        println!("Result2: {:?}", result2);
        assert!(result1.is_err());
        assert!(result2.is_err());
    }

    #[test]
    fn it_fails_duplicate_keys() {
        let result = ComponentTemplate::from_str("root { key1: 5, key1: 10 }\n");

        println!("Result: {:?}", result);
        assert!(result.is_err());
    }
}
