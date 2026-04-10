use std::collections::HashMap;
use std::sync::LazyLock as Lazy;
use std::sync::Mutex;

static CMETA_STACK: Lazy<Mutex<Option<CMetaStack>>> = Lazy::new(|| Mutex::new(None));

#[derive(Debug, Clone)]
enum CMetaValue {
    Single(String),
    List(Vec<String>),
}

/// A simple stack-based context system for passing information between macros.
#[derive(Debug, Clone)]
pub struct CMetaStack {
    data: HashMap<String, CMetaValue>,
    parent: Option<Box<CMetaStack>>,
}

impl CMetaStack {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            parent: None,
        }
    }

    pub fn push(key: &str, value: &str) {
        let mut stack = CMETA_STACK.lock().unwrap();
        let mut new_level = Self::new();

        new_level
            .data
            .insert(key.to_string(), CMetaValue::Single(value.to_string()));

        if let Some(current) = stack.take() {
            new_level.parent = Some(Box::new(current));
        }

        *stack = Some(new_level);
    }

    pub fn push_list<I>(key: &str, values: I)
    where
        I: IntoIterator<Item = String>,
    {
        let mut stack = CMETA_STACK.lock().unwrap();
        let mut new_level = Self::new();

        new_level.data.insert(
            key.to_string(),
            CMetaValue::List(values.into_iter().collect()),
        );

        if let Some(current) = stack.take() {
            new_level.parent = Some(Box::new(current));
        }

        *stack = Some(new_level);
    }

    /// Get a value from the stack by key
    ///
    /// This will search the current level and all parent levels
    /// until a value is found or the stack is exhausted.
    pub fn get(key: &str) -> Option<String> {
        let stack = CMETA_STACK.lock().unwrap();

        if let Some(current) = stack.as_ref() {
            current.get_recursive(key)
        } else {
            None
        }
    }

    pub fn get_list(key: &str) -> Option<Vec<String>> {
        let stack = CMETA_STACK.lock().unwrap();

        if let Some(current) = stack.as_ref() {
            current.get_list_recursive(key)
        } else {
            None
        }
    }

    /// Recursive helper for getting values from the stack
    fn get_recursive(&self, key: &str) -> Option<String> {
        if let Some(CMetaValue::Single(value)) = self.data.get(key) {
            return Some(value.clone());
        }

        self.parent
            .as_ref()
            .and_then(|parent| parent.get_recursive(key))
    }

    fn get_list_recursive(&self, key: &str) -> Option<Vec<String>> {
        if let Some(CMetaValue::List(values)) = self.data.get(key) {
            return Some(values.clone());
        }

        self.parent
            .as_ref()
            .and_then(|parent| parent.get_list_recursive(key))
    }
}
