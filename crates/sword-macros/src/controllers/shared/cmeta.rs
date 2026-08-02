#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::LazyLock as Lazy;
use std::sync::Mutex;

static CMETA_STACK: Lazy<Mutex<Option<CMetaStack>>> = Lazy::new(|| Mutex::new(None));

#[derive(Debug, Clone)]
enum CMetaValue {
    Single(String),
    List(Vec<String>),
}

/// A simple stack-based context system for passing information between macros,
/// scoped by controller kind so different kinds never mix.
#[derive(Debug, Clone)]
pub struct CMetaStack {
    data: HashMap<(String, String), CMetaValue>,
    parent: Option<Box<CMetaStack>>,
}

impl CMetaStack {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            parent: None,
        }
    }

    pub fn push(kind: &str, key: &str, value: &str) {
        let mut stack = CMETA_STACK.lock().unwrap();
        let mut new_level = Self::new();

        new_level.data.insert(
            (kind.to_string(), key.to_string()),
            CMetaValue::Single(value.to_string()),
        );

        if let Some(current) = stack.take() {
            new_level.parent = Some(Box::new(current));
        }

        *stack = Some(new_level);
    }

    pub fn push_list<I>(kind: &str, key: &str, values: I)
    where
        I: IntoIterator<Item = String>,
    {
        let mut stack = CMETA_STACK.lock().unwrap();
        let mut new_level = Self::new();

        new_level.data.insert(
            (kind.to_string(), key.to_string()),
            CMetaValue::List(values.into_iter().collect()),
        );

        if let Some(current) = stack.take() {
            new_level.parent = Some(Box::new(current));
        }

        *stack = Some(new_level);
    }

    /// Get a value from the stack by kind and key
    ///
    /// This will search the current level and all parent levels
    /// until a value is found or the stack is exhausted.
    pub fn get(kind: &str, key: &str) -> Option<String> {
        let stack = CMETA_STACK.lock().unwrap();

        if let Some(current) = stack.as_ref() {
            current.get_recursive(kind, key)
        } else {
            None
        }
    }

    pub fn get_list(kind: &str, key: &str) -> Option<Vec<String>> {
        let stack = CMETA_STACK.lock().unwrap();

        if let Some(current) = stack.as_ref() {
            current.get_list_recursive(kind, key)
        } else {
            None
        }
    }

    /// Recursive helper for getting values from the stack
    fn get_recursive(&self, kind: &str, key: &str) -> Option<String> {
        if let Some(CMetaValue::Single(value)) = self.data.get(&(kind.to_string(), key.to_string()))
        {
            return Some(value.clone());
        }

        self.parent
            .as_ref()
            .and_then(|parent| parent.get_recursive(kind, key))
    }

    fn get_list_recursive(&self, kind: &str, key: &str) -> Option<Vec<String>> {
        if let Some(CMetaValue::List(values)) = self.data.get(&(kind.to_string(), key.to_string()))
        {
            return Some(values.clone());
        }

        self.parent
            .as_ref()
            .and_then(|parent| parent.get_list_recursive(kind, key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_are_isolated() {
        CMetaStack::push("iso_web", "controller_name", "Users");
        CMetaStack::push("iso_event", "controller_name", "Mail");
        CMetaStack::push("iso_socket", "namespace", "/chat");

        assert_eq!(
            CMetaStack::get("iso_web", "controller_name"),
            Some("Users".to_string())
        );
        assert_eq!(
            CMetaStack::get("iso_event", "controller_name"),
            Some("Mail".to_string())
        );
        assert_eq!(
            CMetaStack::get("iso_socket", "namespace"),
            Some("/chat".to_string())
        );
        assert_eq!(CMetaStack::get("iso_web", "namespace"), None);
        assert_eq!(CMetaStack::get("iso_socket", "controller_name"), None);
        assert_eq!(CMetaStack::get("iso_event", "namespace"), None);
    }

    #[test]
    fn latest_push_wins_per_kind() {
        CMetaStack::push("win_web", "controller_name", "First");
        CMetaStack::push("win_event", "controller_name", "Mail");
        CMetaStack::push("win_web", "controller_name", "Second");

        assert_eq!(
            CMetaStack::get("win_web", "controller_name"),
            Some("Second".to_string())
        );
        assert_eq!(
            CMetaStack::get("win_event", "controller_name"),
            Some("Mail".to_string())
        );
    }
}
