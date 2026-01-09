use std::fmt;
use crate::domain::http_request::HttpResponse;

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
                        message: format!(
                            "Expected {} {:?} {}",
                            path, op, value
                        ),
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

