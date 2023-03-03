use toml_edit::{visit::*, Document, Item, Value};

#[derive(Default)]
pub(crate) struct DeepKeysCollector<'doc> {
    current_path: Vec<&'doc str>,
    pub keys: Vec<String>,
}

impl DeepKeysCollector<'_> {
    pub fn get_keys(toml_string: String) -> Vec<String> {
        let document: Document = toml_string.parse().unwrap();
        let mut visitor = DeepKeysCollector::default();
        visitor.visit_document(&document);

        visitor.keys.dedup();
        visitor.keys.sort();
        visitor.keys
    }
}

impl<'doc> Visit<'doc> for DeepKeysCollector<'doc> {
    fn visit_table_like_kv(&mut self, key: &'doc str, node: &'doc Item) {
        self.current_path.push(key);
        self.visit_item(node);
        self.current_path.pop();
    }

    fn visit_value(&mut self, node: &'doc Value) {
        match node {
            Value::InlineTable(_table) => {}
            _ => {
                self.keys.push(self.current_path.join("."));
            }
        };

        match node {
            Value::String(s) => self.visit_string(s),
            Value::Integer(i) => self.visit_integer(i),
            Value::Float(f) => self.visit_float(f),
            Value::Boolean(b) => self.visit_boolean(b),
            Value::Datetime(dt) => self.visit_datetime(dt),
            Value::Array(array) => self.visit_array(array),
            Value::InlineTable(table) => self.visit_inline_table(table),
        }
    }
}

#[cfg(test)]
mod tests {
    use toml_edit::Document;

    use crate::settings::Settings;

    use super::*;

    #[test]
    fn test_basic() {
        let input = r#"
laputa = "sky-castle"
the-force = { value = "surrounds-you" }
"#;

        let document: Document = input.parse().unwrap();
        let mut visitor = DeepKeysCollector::default();
        visitor.visit_document(&document);

        assert_eq!(visitor.current_path, Vec::<&str>::new());
        assert_eq!(visitor.keys, vec!["laputa", "the-force.value"]);
    }

    #[test]
    fn test_default_config() {
        let input = toml::to_string_pretty(&Settings::new().unwrap()).unwrap();

        let document: Document = input.parse().unwrap();
        let mut visitor = DeepKeysCollector::default();
        visitor.visit_document(&document);

        assert_eq!(visitor.current_path, Vec::<&str>::new());
        visitor.keys.dedup();
        visitor.keys.sort();
        assert_eq!(
            visitor.keys,
            vec![
                "allow_amend",
                "file_ignore",
                "model_provider",
                "openai.api_key",
                "openai.model",
                "openai.retries",
                "output.lang",
                "prompt.commit_summary",
                "prompt.commit_title",
                "prompt.file_diff",
                "prompt.translation",
            ]
        );
    }

    #[test]
    fn test_get_keys() {
        let input = toml::to_string_pretty(&Settings::new().unwrap()).unwrap();

        assert_eq!(
            DeepKeysCollector::get_keys(input),
            vec![
                "allow_amend",
                "file_ignore",
                "model_provider",
                "openai.api_key",
                "openai.model",
                "openai.retries",
                "output.lang",
                "prompt.commit_summary",
                "prompt.commit_title",
                "prompt.file_diff",
                "prompt.translation",
            ]
        );
    }
}
