use std::collections::HashMap;

pub fn lookup_search_replace() -> Result<HashMap<String, String>, std::io::Error> {
    let mut h = HashMap::new();
    h.insert("scala".to_string(), "[[scala]]".to_string());
    h.insert("rust".to_string(), "[[rust]]".to_string());
    Ok(h)
}
