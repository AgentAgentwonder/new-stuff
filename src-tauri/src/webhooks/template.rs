use super::types::WebhookError;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

pub struct TemplateEngine {
    variable_pattern: Regex,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            variable_pattern: Regex::new(r"\$\{(\w+)\}").unwrap(),
        }
    }

    pub fn render(
        &self,
        template: &str,
        variables: &HashMap<String, Value>,
    ) -> Result<String, WebhookError> {
        let mut result = template.to_string();

        for cap in self.variable_pattern.captures_iter(template) {
            let full_match = &cap[0];
            let var_name = &cap[1];

            let replacement = match variables.get(var_name) {
                Some(value) => match value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => "null".to_string(),
                    Value::Array(_) | Value::Object(_) => {
                        serde_json::to_string(value).map_err(WebhookError::Serialization)?
                    }
                },
                None => {
                    return Err(WebhookError::InvalidTemplate(format!(
                        "Variable '{}' not found",
                        var_name
                    )))
                }
            };

            result = result.replace(full_match, &replacement);
        }

        Ok(result)
    }

    pub fn extract_variables(&self, template: &str) -> Vec<String> {
        self.variable_pattern
            .captures_iter(template)
            .map(|cap| cap[1].to_string())
            .collect()
    }

    pub fn validate(
        &self,
        template: &str,
        available_variables: &[String],
    ) -> Result<(), WebhookError> {
        let used_variables = self.extract_variables(template);

        for var in used_variables {
            if !available_variables.contains(&var) {
                return Err(WebhookError::InvalidTemplate(format!(
                    "Variable '{}' is not available. Available: {:?}",
                    var, available_variables
                )));
            }
        }

        Ok(())
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_variable_replacement() {
        let engine = TemplateEngine::new();
        let template = "Hello ${name}!";
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), json!("World"));

        let result = engine.render(template, &vars).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_multiple_variables() {
        let engine = TemplateEngine::new();
        let template = "Token: ${token}, Price: $${price}, Volume: ${volume}";
        let mut vars = HashMap::new();
        vars.insert("token".to_string(), json!("SOL"));
        vars.insert("price".to_string(), json!(150.75));
        vars.insert("volume".to_string(), json!(1000000));

        let result = engine.render(template, &vars).unwrap();
        assert_eq!(result, "Token: SOL, Price: $150.75, Volume: 1000000");
    }

    #[test]
    fn test_json_body_template() {
        let engine = TemplateEngine::new();
        let template = r#"{"symbol":"${symbol}","price":${price},"change":${change}}"#;
        let mut vars = HashMap::new();
        vars.insert("symbol".to_string(), json!("SOL/USDT"));
        vars.insert("price".to_string(), json!(150.25));
        vars.insert("change".to_string(), json!(5.5));

        let result = engine.render(template, &vars).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["symbol"], "SOL/USDT");
        assert_eq!(parsed["price"], 150.25);
    }

    #[test]
    fn test_missing_variable() {
        let engine = TemplateEngine::new();
        let template = "Hello ${name}!";
        let vars = HashMap::new();

        let result = engine.render(template, &vars);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_variables() {
        let engine = TemplateEngine::new();
        let template = "Token: ${token}, Price: ${price}, ${token} again";
        let vars = engine.extract_variables(template);

        assert!(vars.contains(&"token".to_string()));
        assert!(vars.contains(&"price".to_string()));
    }

    #[test]
    fn test_validate_template() {
        let engine = TemplateEngine::new();
        let template = "Token: ${token}, Price: ${price}";
        let available = vec![
            "token".to_string(),
            "price".to_string(),
            "volume".to_string(),
        ];

        assert!(engine.validate(template, &available).is_ok());
    }

    #[test]
    fn test_validate_template_failure() {
        let engine = TemplateEngine::new();
        let template = "Token: ${token}, Price: ${price}";
        let available = vec!["token".to_string()];

        assert!(engine.validate(template, &available).is_err());
    }
}
