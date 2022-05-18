use rustc_hash::FxHashMap as HashMap;

use regex::{Regex};
pub struct RegexCache {
    cache: HashMap<String, Regex>,
}

impl RegexCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::default(),
        }
    }

    pub fn get(&self, str: &String) -> Option<&Regex> {
        match self.cache.get(str) {
            Some(re) => Some(re),
            None => None
        }
    }

    pub fn put(&mut self, regex: &String) -> Option<Regex>{
        match Regex::new(regex) {
            Ok(re) => {
                self.cache.insert(regex.clone(), re.clone());
                Some(re)
            }
            Err(_) => None
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_regex() {
        let r_cache = RegexCache::new();
        let r = r_cache.get(&".*".to_string());
        assert!(r.is_none());
    }

    #[test]
    fn put_regex() {
        let mut r_cache = RegexCache::new();
        r_cache.put(&".*".to_string());
        let r = r_cache.get(&".*".to_string());
        assert!(r.is_some());
    }

}