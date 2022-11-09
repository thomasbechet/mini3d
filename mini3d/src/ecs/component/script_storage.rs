use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum NodeValue {
    Null,
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
    Node(HashMap<String, NodeValue>),
}

#[derive(Serialize, Deserialize)]
pub struct ScriptStorageComponent {
    root: NodeValue,
}

impl Default for ScriptStorageComponent {
    fn default() -> Self {
        Self { root: NodeValue::Node(Default::default()) }
    }
}

impl ScriptStorageComponent {

    fn find_node(&self, key: &str) -> Option<&NodeValue> {
        let mut current = &self.root;
        for split in key.split('.') {
            match current {
                NodeValue::Node(childs) => {
                    match childs.get(split) {
                        Some(child) => { current = child; }
                        _ => { return None; }
                    }
                },
                _ => { return None; }
            }
        }
        Some(current)
    }

    fn insert_node(&mut self, key: &str, node: NodeValue) {
        // Start with root
        let mut current = &mut self.root;
        // Iterate over each token
        for token in key.split('.') {
            // Lazy hashmap creation
            match *current {
                NodeValue::Node(_) => {}
                _ => { *current = NodeValue::Node(Default::default()); }
            }
            match current {
                NodeValue::Node(childs) => {
                    // Insert new node
                    current = childs.entry(token.to_string()).or_insert(NodeValue::Null);
                },
                _ => {
                    panic!("Corrupted storage structure");
                }
            }
        }
        // Insert node value
        *current = node;
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.find_node(key).and_then(|node| match node {
            NodeValue::Bool(value) => Some(*value),
            _ => None,
        })
    }

    pub fn set_bool(&mut self, key: &str, value: bool) {
        self.insert_node(key, NodeValue::Bool(value));
    }

    pub fn get_int(&self, key: &str) -> Option<i32> {
        self.find_node(key).and_then(|node| match node {
            NodeValue::Int(value) => Some(*value),
            _ => None,
        })
    }

    pub fn set_int(&mut self, key: &str, value: i32) {
        self.insert_node(key, NodeValue::Int(value));
    }

    pub fn get_float(&self, key: &str) -> Option<f32> {
        self.find_node(key).and_then(|node| match node {
            NodeValue::Float(value) => Some(*value),
            _ => None,
        })
    }

    pub fn set_float(&mut self, key: &str, value: f32) {
        self.insert_node(key, NodeValue::Float(value));
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.find_node(key).and_then(|node| match node {
            NodeValue::String(value) => Some(value.clone()),
            _ => None,
        })
    }

    pub fn set_string(&mut self, key: &str, value: String) {
        self.insert_node(key, NodeValue::String(value));
    }

    pub fn list_keys(&self, key: &str) -> Option<impl Iterator<Item = &String>> {
        self.find_node(key).and_then(|node| match node {
            NodeValue::Node(childs) => {
                Some(childs.keys())
            },
            _ => None,
        })
    }
}