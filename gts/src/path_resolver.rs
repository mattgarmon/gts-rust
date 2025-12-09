use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonPathResolver {
    pub gts_id: String,
    pub content: Value,
    pub path: String,
    pub value: Option<Value>,
    pub resolved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_fields: Option<Vec<String>>,
}

impl JsonPathResolver {
    pub fn new(gts_id: String, content: Value) -> Self {
        JsonPathResolver {
            gts_id,
            content,
            path: String::new(),
            value: None,
            resolved: false,
            error: None,
            available_fields: None,
        }
    }

    fn normalize(&self, path: &str) -> String {
        path.replace('/', ".")
    }

    fn split_raw_parts(&self, norm: &str) -> Vec<String> {
        norm.split('.')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    fn parse_part(&self, seg: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut buf = String::new();
        let mut i = 0;
        let chars: Vec<char> = seg.chars().collect();

        while i < chars.len() {
            let ch = chars[i];
            if ch == '[' {
                if !buf.is_empty() {
                    out.push(buf.clone());
                    buf.clear();
                }
                if let Some(j) = seg[i + 1..].find(']') {
                    let j = i + 1 + j;
                    out.push(seg[i..=j].to_string());
                    i = j + 1;
                } else {
                    buf.push_str(&seg[i..]);
                    break;
                }
            } else {
                buf.push(ch);
                i += 1;
            }
        }

        if !buf.is_empty() {
            out.push(buf);
        }

        out
    }

    fn parts(&self, path: &str) -> Vec<String> {
        let norm = self.normalize(path);
        let raw = self.split_raw_parts(&norm);
        let mut parts = Vec::new();

        for seg in raw {
            parts.extend(self.parse_part(&seg));
        }

        parts
    }

    fn list_available(node: &Value, prefix: &str, out: &mut Vec<String>) {
        match node {
            Value::Object(map) => {
                for (k, v) in map {
                    let p = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", prefix, k)
                    };
                    out.push(p.clone());
                    if v.is_object() || v.is_array() {
                        Self::list_available(v, &p, out);
                    }
                }
            }
            Value::Array(arr) => {
                for (i, v) in arr.iter().enumerate() {
                    let p = if prefix.is_empty() {
                        format!("[{}]", i)
                    } else {
                        format!("{}[{}]", prefix, i)
                    };
                    out.push(p.clone());
                    if v.is_object() || v.is_array() {
                        Self::list_available(v, &p, out);
                    }
                }
            }
            _ => {}
        }
    }

    fn collect_from(&self, node: &Value) -> Vec<String> {
        let mut acc = Vec::new();
        Self::list_available(node, "", &mut acc);
        acc
    }

    pub fn resolve(mut self, path: &str) -> Self {
        self.path = path.to_string();
        self.value = None;
        self.resolved = false;
        self.error = None;
        self.available_fields = None;

        let parts = self.parts(path);
        let mut cur = self.content.clone();

        for p in parts {
            match &cur {
                Value::Array(arr) => {
                    let idx = if p.starts_with('[') && p.ends_with(']') {
                        let idx_str = &p[1..p.len() - 1];
                        match idx_str.parse::<usize>() {
                            Ok(i) => i,
                            Err(_) => {
                                self.error =
                                    Some(format!("Expected list index at segment '{}'", p));
                                self.available_fields = Some(self.collect_from(&cur));
                                return self;
                            }
                        }
                    } else {
                        match p.parse::<usize>() {
                            Ok(i) => i,
                            Err(_) => {
                                self.error =
                                    Some(format!("Expected list index at segment '{}'", p));
                                self.available_fields = Some(self.collect_from(&cur));
                                return self;
                            }
                        }
                    };

                    if idx >= arr.len() {
                        self.error = Some(format!("Index out of range at segment '{}'", p));
                        self.available_fields = Some(self.collect_from(&cur));
                        return self;
                    }

                    cur = arr[idx].clone();
                }
                Value::Object(map) => {
                    if p.starts_with('[') && p.ends_with(']') {
                        self.error = Some(format!(
                            "Path not found at segment '{}' in '{}', see available fields",
                            p, path
                        ));
                        self.available_fields = Some(self.collect_from(&cur));
                        return self;
                    }

                    if let Some(v) = map.get(&p) {
                        cur = v.clone();
                    } else {
                        self.error = Some(format!(
                            "Path not found at segment '{}' in '{}', see available fields",
                            p, path
                        ));
                        self.available_fields = Some(self.collect_from(&cur));
                        return self;
                    }
                }
                _ => {
                    self.error = Some(format!("Cannot descend into {:?} at segment '{}'", cur, p));
                    self.available_fields = if cur.is_object() || cur.is_array() {
                        Some(self.collect_from(&cur))
                    } else {
                        Some(Vec::new())
                    };
                    return self;
                }
            }
        }

        self.value = Some(cur);
        self.resolved = true;
        self
    }

    pub fn failure(mut self, path: &str, error: &str) -> Self {
        self.path = path.to_string();
        self.value = None;
        self.resolved = false;
        self.error = Some(error.to_string());
        self.available_fields = Some(Vec::new());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_resolve_simple_path() {
        let content = json!({"field": "value"});
        let resolver = JsonPathResolver::new("gts.test.v1~".to_string(), content);
        let result = resolver.resolve("field");
        assert!(result.resolved);
        assert_eq!(result.value, Some(Value::String("value".to_string())));
    }

    #[test]
    fn test_resolve_nested_path() {
        let content = json!({"outer": {"inner": "value"}});
        let resolver = JsonPathResolver::new("gts.test.v1~".to_string(), content);
        let result = resolver.resolve("outer.inner");
        assert!(result.resolved);
        assert_eq!(result.value, Some(Value::String("value".to_string())));
    }

    #[test]
    fn test_resolve_array_index() {
        let content = json!({"items": [1, 2, 3]});
        let resolver = JsonPathResolver::new("gts.test.v1~".to_string(), content);
        let result = resolver.resolve("items[1]");
        assert!(result.resolved);
        assert_eq!(result.value, Some(Value::Number(2.into())));
    }

    #[test]
    fn test_resolve_missing_path() {
        let content = json!({"field": "value"});
        let resolver = JsonPathResolver::new("gts.test.v1~".to_string(), content);
        let result = resolver.resolve("missing");
        assert!(!result.resolved);
        assert!(result.error.is_some());
    }
}
