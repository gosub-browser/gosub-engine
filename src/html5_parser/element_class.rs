use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ElementClass {
    /// a map of classes applied to an HTML element.
    /// key = name, value = is_active
    /// the is_active is used to toggle a class (JavaScript API)
    class_map: HashMap<String, bool>,
}

impl Default for ElementClass {
    fn default() -> Self {
        Self::new()
    }
}

impl ElementClass {
    /// Initialise a new (empty) ElementClass
    pub fn new() -> Self {
        ElementClass {
            class_map: HashMap::new(),
        }
    }

    /// Check if class name exists
    pub fn contains(&self, name: &str) -> bool {
        self.class_map.contains_key(name)
    }

    /// Add a new class (if already exists, does nothing)
    pub fn add(&mut self, name: &str) {
        // by default, adding a new class will be active.
        // however, map.insert will update a key if it exists
        // and we don't want to overwrite an inactive class to make it active unintentionally
        // so we ignore this operation if the class already exists
        if !self.contains(name) {
            self.class_map.insert(name.to_owned(), true);
        }
    }

    /// Remove a class (does nothing if not exists)
    pub fn remove(&mut self, name: &str) {
        self.class_map.remove(name);
    }

    /// Toggle a class active/inactive. Does nothing if class doesn't exist
    pub fn toggle(&mut self, name: &str) {
        if let Some(is_active) = self.class_map.get_mut(name) {
            *is_active = !*is_active;
        }
    }

    /// Set explicitly if a class is active or not. Does nothing if class doesn't exist
    pub fn set_active(&mut self, name: &str, is_active: bool) {
        if let Some(is_active_item) = self.class_map.get_mut(name) {
            *is_active_item = is_active;
        }
    }

    /// Check if a class is active. Returns false if class doesn't exist
    pub fn is_active(&self, name: &str) -> bool {
        if let Some(is_active) = self.class_map.get(name) {
            return *is_active;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_nonexistant_class() {
        let classes = ElementClass::new();
        assert_eq!(classes.contains("nope"), false);
    }

    #[test]
    fn contains_valid_class() {
        let mut classes = ElementClass::new();
        classes.add("yep");
        assert_eq!(classes.contains("yep"), true);
    }
}
