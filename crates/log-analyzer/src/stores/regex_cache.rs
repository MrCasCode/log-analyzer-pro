use std::collections::HashMap;

use regex::Regex;
use std::sync::RwLock;

struct RegexCache {
    cache: HashMap<String, Regex>,
}

impl RegexCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn get(&mut self, str: &String) -> Option<Regex> {
        if self.cache.get(str).is_none() {
            match Regex::new(str) {
                Ok(re) => {
                    self.cache.insert(str.clone(), re);
                }
                Err(_) => {}
            }
        }
        Some(self.cache.get(str).unwrap().clone())
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn return_new_regex() {
        let mut r_cache = RegexCache::new();
        let r = r_cache.get(&".*".to_string());
        assert!(r.is_some());
    }

}