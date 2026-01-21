use crate::domain::http_request::HttpResponse;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Assertion {
    Binary {
        path: String,
        op: Operator,
        value: Value,
    },
    In {
        path: String,
        values: Vec<Value>,
    },
    Between {
        path: String,
        min: Value,
        max: Value,
    },
    Exists {
        path: String,
    },
    Unary {
        path: String,
    },
}

#[derive(Debug)]
pub struct AssertionFailure {
    pub path: String,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub message: String,
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Eq,
    Ne,
    Gt,
    Lt,
    Gte,
    Lte,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    String(String),
    Number(i64),
    Bool(bool),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Number(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
        }
    }
}

impl fmt::Display for AssertionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.expected, &self.actual) {
            (Some(expected), Some(actual)) => {
                write!(
                    f,
                    "{}\n  expected: {}\n  actual:   {}",
                    self.message, expected, actual
                )
            }
            (Some(expected), None) => {
                write!(
                    f,
                    "{}\n  expected: {}\n  actual:   <missing>",
                    self.message, expected
                )
            }
            _ => write!(f, "{}", self.message),
        }
    }
}

fn resolve_path(response: &HttpResponse, path: &str) -> Option<Value> {
    // status
    if path == "status" {
        return Some(Value::Number(response.status as i64));
    }

    // full body as string
    if path == "body" {
        return response.body.as_ref().map(|s| Value::String(s.clone()));
    }

    // body.xxx.yyy → only if JSON
    if let Some(rest) = path.strip_prefix("body.") {
        let body_str = response.body.as_ref()?; // &String

        // Try parse JSON
        let json: serde_json::Value = serde_json::from_str(body_str).ok()?; // parse failure → None

        let mut current = &json;
        for key in rest.split('.') {
            current = current.get(key)?;
        }

        return match current {
            serde_json::Value::String(s) => Some(Value::String(s.clone())),
            serde_json::Value::Number(n) => Some(Value::Number(n.as_i64()?)),
            serde_json::Value::Bool(b) => Some(Value::Bool(*b)),
            _ => None,
        };
    }

    None
}

fn compare(op: &Operator, actual: &Value, expected: &Value) -> bool {
    match (op, actual, expected) {
        (Operator::Eq, a, b) => a == b,
        (Operator::Ne, a, b) => a != b,

        (Operator::Gt, Value::Number(a), Value::Number(b)) => a > b,
        (Operator::Lt, Value::Number(a), Value::Number(b)) => a < b,
        (Operator::Gte, Value::Number(a), Value::Number(b)) => a >= b,
        (Operator::Lte, Value::Number(a), Value::Number(b)) => a <= b,

        _ => false,
    }
}

impl Assertion {
    pub fn check(&self, response: &HttpResponse) -> Result<(), AssertionFailure> {
        match self {
            Assertion::Binary { path, op, value } => {
                let actual = match resolve_path(response, path) {
                    Some(v) => v,
                    None => {
                        return Err(AssertionFailure {
                            path: path.clone(),
                            expected: Some(value.to_string()),
                            actual: None,
                            message: format!("Path '{}' not found", path),
                        });
                    }
                };

                if !compare(op, &actual, value) {
                    return Err(AssertionFailure {
                        path: path.clone(),
                        expected: Some(value.to_string()),
                        actual: Some(actual.to_string()),
                        message: format!("Expected {} {:?} {}", path, op, value),
                    });
                }
            }

            Assertion::Exists { path } => {
                if resolve_path(response, path).is_none() {
                    return Err(AssertionFailure {
                        path: path.clone(),
                        expected: Some("exists".into()),
                        actual: None,
                        message: format!("Expected '{}' to exist", path),
                    });
                }
            }

            Assertion::Unary { path } => {
                let actual = match resolve_path(response, path) {
                    Some(v) => v,
                    None => {
                        return Err(AssertionFailure {
                            path: path.clone(),
                            expected: Some("true".into()),
                            actual: None,
                            message: format!("Path '{}' not found", path),
                        });
                    }
                };

                if actual != Value::Bool(true) {
                    return Err(AssertionFailure {
                        path: path.clone(),
                        expected: Some("true".into()),
                        actual: Some(actual.to_string()),
                        message: format!("Expected '{}' to be true", path),
                    });
                }
            }

            Assertion::In { path, values } => {
                let actual = match resolve_path(response, path) {
                    Some(v) => v,
                    None => {
                        return Err(AssertionFailure {
                            path: path.clone(),
                            expected: Some(format!("{:?}", values)),
                            actual: None,
                            message: format!("Path '{}' not found", path),
                        });
                    }
                };

                if !values.contains(&actual) {
                    return Err(AssertionFailure {
                        path: path.clone(),
                        expected: Some(format!("{:?}", values)),
                        actual: Some(actual.to_string()),
                        message: format!("Expected '{}' to be in list", path),
                    });
                }
            }

            Assertion::Between { path, min, max } => {
                let actual = match resolve_path(response, path) {
                    Some(v) => v,
                    None => {
                        return Err(AssertionFailure {
                            path: path.clone(),
                            expected: Some(format!("between {} and {}", min, max)),
                            actual: None,
                            message: format!("Path '{}' not found", path),
                        });
                    }
                };

                match (&actual, min, max) {
                    (Value::Number(a), Value::Number(lo), Value::Number(hi))
                        if a >= lo && a <= hi => {}
                    _ => {
                        return Err(AssertionFailure {
                            path: path.clone(),
                            expected: Some(format!("between {} and {}", min, max)),
                            actual: Some(actual.to_string()),
                            message: format!("Value not in range"),
                        });
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_response(status: u16, body: Option<&str>) -> HttpResponse {
        HttpResponse {
            request: None,
            duration: std::time::Duration::from_millis(100),
            status,
            headers: HashMap::new(),
            body: body.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_resolve_path_status() {
        let response = create_response(200, None);
        assert_eq!(resolve_path(&response, "status"), Some(Value::Number(200)));
    }

    #[test]
    fn test_resolve_path_body() {
        let response = create_response(200, Some("hello world"));
        assert_eq!(
            resolve_path(&response, "body"),
            Some(Value::String("hello world".to_string()))
        );
    }

    #[test]
    fn test_resolve_path_body_json_string() {
        let body = r#"{"name": "test", "active": true, "count": 42}"#;
        let response = create_response(200, Some(body));
        assert_eq!(
            resolve_path(&response, "body.name"),
            Some(Value::String("test".to_string()))
        );
        assert_eq!(
            resolve_path(&response, "body.active"),
            Some(Value::Bool(true))
        );
        assert_eq!(
            resolve_path(&response, "body.count"),
            Some(Value::Number(42))
        );
    }

    #[test]
    fn test_resolve_path_body_json_nested() {
        let body = r#"{"user": {"name": "alice", "age": 30}}"#;
        let response = create_response(200, Some(body));
        assert_eq!(
            resolve_path(&response, "body.user.name"),
            Some(Value::String("alice".to_string()))
        );
        assert_eq!(
            resolve_path(&response, "body.user.age"),
            Some(Value::Number(30))
        );
    }

    #[test]
    fn test_resolve_path_missing() {
        let response = create_response(200, Some("{}"));
        assert_eq!(resolve_path(&response, "body.missing"), None);
        assert_eq!(resolve_path(&response, "invalid"), None);
    }

    #[test]
    fn test_compare_eq() {
        assert!(compare(&Operator::Eq, &Value::Number(5), &Value::Number(5)));
        assert!(!compare(
            &Operator::Eq,
            &Value::Number(5),
            &Value::Number(6)
        ));
        assert!(compare(
            &Operator::Eq,
            &Value::String("a".to_string()),
            &Value::String("a".to_string())
        ));
        assert!(compare(
            &Operator::Eq,
            &Value::Bool(true),
            &Value::Bool(true)
        ));
    }

    #[test]
    fn test_compare_ne() {
        assert!(!compare(
            &Operator::Ne,
            &Value::Number(5),
            &Value::Number(5)
        ));
        assert!(compare(&Operator::Ne, &Value::Number(5), &Value::Number(6)));
    }

    #[test]
    fn test_compare_gt_lt() {
        assert!(compare(&Operator::Gt, &Value::Number(6), &Value::Number(5)));
        assert!(!compare(
            &Operator::Gt,
            &Value::Number(5),
            &Value::Number(5)
        ));
        assert!(compare(&Operator::Lt, &Value::Number(4), &Value::Number(5)));
        assert!(compare(
            &Operator::Gte,
            &Value::Number(5),
            &Value::Number(5)
        ));
        assert!(compare(
            &Operator::Lte,
            &Value::Number(5),
            &Value::Number(5)
        ));
        // Non-numbers should return false
        assert!(!compare(
            &Operator::Gt,
            &Value::String("a".to_string()),
            &Value::String("b".to_string())
        ));
    }

    #[test]
    fn test_assertion_binary_pass() {
        let assertion = Assertion::Binary {
            path: "status".to_string(),
            op: Operator::Eq,
            value: Value::Number(200),
        };
        let response = create_response(200, None);
        assert!(assertion.check(&response).is_ok());
    }

    #[test]
    fn test_assertion_binary_fail() {
        let assertion = Assertion::Binary {
            path: "status".to_string(),
            op: Operator::Eq,
            value: Value::Number(404),
        };
        let response = create_response(200, None);
        let result = assertion.check(&response);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.path, "status");
        assert_eq!(err.expected, Some("404".to_string()));
        assert_eq!(err.actual, Some("200".to_string()));
    }

    #[test]
    fn test_assertion_binary_path_missing() {
        let assertion = Assertion::Binary {
            path: "body.missing".to_string(),
            op: Operator::Eq,
            value: Value::Number(42),
        };
        let response = create_response(200, Some("{}"));
        let result = assertion.check(&response);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.path, "body.missing");
        assert_eq!(err.actual, None);
    }

    #[test]
    fn test_assertion_exists_pass() {
        let assertion = Assertion::Exists {
            path: "status".to_string(),
        };
        let response = create_response(200, None);
        assert!(assertion.check(&response).is_ok());
    }

    #[test]
    fn test_assertion_exists_fail() {
        let assertion = Assertion::Exists {
            path: "body.missing".to_string(),
        };
        let response = create_response(200, Some("{}"));
        let result = assertion.check(&response);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.path, "body.missing");
    }

    #[test]
    fn test_assertion_unary_pass() {
        let assertion = Assertion::Unary {
            path: "body.active".to_string(),
        };
        let response = create_response(200, Some(r#"{"active": true}"#));
        assert!(assertion.check(&response).is_ok());
    }

    #[test]
    fn test_assertion_unary_fail() {
        let assertion = Assertion::Unary {
            path: "body.active".to_string(),
        };
        let response = create_response(200, Some(r#"{"active": false}"#));
        let result = assertion.check(&response);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.expected, Some("true".to_string()));
        assert_eq!(err.actual, Some("false".to_string()));
    }

    #[test]
    fn test_assertion_in_pass() {
        let assertion = Assertion::In {
            path: "status".to_string(),
            values: vec![Value::Number(200), Value::Number(201)],
        };
        let response = create_response(200, None);
        assert!(assertion.check(&response).is_ok());
    }

    #[test]
    fn test_assertion_in_fail() {
        let assertion = Assertion::In {
            path: "status".to_string(),
            values: vec![Value::Number(201), Value::Number(202)],
        };
        let response = create_response(200, None);
        let result = assertion.check(&response);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.path, "status");
    }

    #[test]
    fn test_assertion_between_pass() {
        let assertion = Assertion::Between {
            path: "status".to_string(),
            min: Value::Number(199),
            max: Value::Number(300),
        };
        let response = create_response(200, None);
        assert!(assertion.check(&response).is_ok());
    }

    #[test]
    fn test_assertion_between_fail() {
        let assertion = Assertion::Between {
            path: "status".to_string(),
            min: Value::Number(300),
            max: Value::Number(400),
        };
        let response = create_response(200, None);
        let result = assertion.check(&response);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.path, "status");
    }
}
