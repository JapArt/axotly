use anyhow::bail;
use anyhow::{Context, Result};
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;
use url::Url;

use crate::domain::assertion::{Operator, Value};
use crate::domain::http_request::{Body, HttpRequest};
use crate::domain::{Assertion, TestCase};

#[derive(Parser)]
#[grammar = "src/parser/grammar.pest"]
pub struct AxParser;

impl AxParser {
    /// Parse a full .ax file from its contents
    pub fn parse_file(file: &String) -> Result<Vec<TestCase>> {
        // Parse the file content using Pest
        let mut pairs = AxParser::parse(Rule::file, file.as_str())
            .map_err(|e| anyhow::anyhow!("Failed to parse input: {}", e))?;

        // There should be exactly one top-level file pair
        let file_pair = pairs
            .next()
            .ok_or_else(|| anyhow::anyhow!("Empty .ax file"))?;

        // Now parse each test_block inside the file
        let mut tests = Vec::new();
        for inner in file_pair.into_inner() {
            if inner.as_rule() == Rule::test_block {
                let test_case = parse_test_block(inner)?;
                tests.push(test_case);
            }
        }

        Ok(tests)
    }
}

pub fn parse_http_request(pair: Pair<Rule>) -> Result<HttpRequest> {
    debug_assert_eq!(pair.as_rule(), Rule::request);

    let mut method: Option<String> = None;
    let mut url: Option<Url> = None;
    let mut headers: HashMap<String, String> = HashMap::new();
    let mut body: Option<Body> = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::method => {
                method = Some(inner.as_str().to_string());
            }
            Rule::url => {
                let url_str = inner.as_str();
                url =
                    Some(Url::parse(url_str).with_context(|| format!("Invalid URL: {}", url_str))?);
            }
            Rule::headers => {
                for header_pair in inner.into_inner() {
                    debug_assert_eq!(header_pair.as_rule(), Rule::header);
                    let mut header_inner = header_pair.into_inner();
                    let key = header_inner.next().unwrap().as_str().to_string();
                    let value = header_inner.next().unwrap().as_str().to_string();
                    headers.insert(key, value);
                }
            }

            Rule::body => {
                for body_inner in inner.into_inner() {
                    if body_inner.as_rule() == Rule::body_content {
                        let text = body_inner.as_str().to_string();
                        body = Some(Body::Text(text));
                    }
                }
            }
            _ => {}
        }
    }

    Ok(HttpRequest {
        method: method.unwrap_or_default(),
        url: url.context("HTTP request missing URL")?,
        headers,
        body,
    })
}

pub fn parse_assertion(pair: Pair<Rule>) -> Result<Assertion> {
    debug_assert_eq!(pair.as_rule(), Rule::expect_expr);

    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::binary_op => parse_binary_op(inner),
        Rule::in_op => parse_in_op(inner),
        Rule::between_op => parse_between_op(inner),
        Rule::exists_op => parse_exists_op(inner),
        Rule::unary_path => parse_unary_path(inner),
        _ => bail!("Unsupported assertion type: {:?}", inner.as_rule()),
    }
}

fn parse_binary_op(pair: Pair<Rule>) -> Result<Assertion> {
    let mut inner = pair.into_inner();

    let path = inner.next().unwrap().as_str().to_string();
    let op = parse_operator(inner.next().unwrap())?;
    let val = parse_value(inner.next().unwrap())?;

    Ok(Assertion::Binary {
        path,
        op,
        value: val,
    })
}

fn parse_operator(pair: Pair<Rule>) -> Result<Operator> {
    Ok(match pair.as_str() {
        "==" => Operator::Eq,
        "!=" => Operator::Ne,
        ">" => Operator::Gt,
        "<" => Operator::Lt,
        ">=" => Operator::Gte,
        "<=" => Operator::Lte,
        _ => bail!("Unknown operator {}", pair.as_str()),
    })
}

fn parse_in_op(pair: Pair<Rule>) -> Result<Assertion> {
    let mut inner = pair.into_inner();

    let path = inner.next().unwrap().as_str().to_string();
    let mut values = Vec::new();

    for p in inner {
        if p.as_rule() == Rule::value {
            values.push(parse_value(p)?);
        }
    }

    Ok(Assertion::In { path, values })
}

fn parse_between_op(pair: Pair<Rule>) -> Result<Assertion> {
    let mut inner = pair.into_inner();

    let path = inner.next().unwrap().as_str().to_string();
    let min = parse_value(inner.next().unwrap())?;
    let max = parse_value(inner.next().unwrap())?;

    Ok(Assertion::Between { path, min, max })
}

fn parse_exists_op(pair: Pair<Rule>) -> Result<Assertion> {
    let path = pair.into_inner().next().unwrap().as_str().to_string();

    Ok(Assertion::Exists { path })
}

fn parse_unary_path(pair: Pair<Rule>) -> Result<Assertion> {
    let path = pair.as_str().to_string();

    Ok(Assertion::Unary { path })
}

fn parse_value(pair: Pair<Rule>) -> Result<Value> {
    match pair.as_rule() {
        Rule::value => parse_value(pair.into_inner().next().unwrap()),
        Rule::quoted_string => {
            let s = pair.as_str();
            Ok(Value::String(s[1..s.len() - 1].to_string()))
        }
        Rule::number => Ok(Value::Number(pair.as_str().parse()?)),
        Rule::boolean => Ok(Value::Bool(pair.as_str() == "true")),
        _ => bail!("Invalid value rule"),
    }
}

pub fn parse_test_block(pair: Pair<Rule>) -> Result<TestCase> {
    debug_assert_eq!(pair.as_rule(), Rule::test_block);
    let mut name: Option<String> = None;
    let mut request: Option<HttpRequest> = None;
    let mut assertions: Vec<Assertion> = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::test_name => {
                name = Some(inner.as_str().to_string());
            }
            Rule::request => {
                request = Some(parse_http_request(inner)?);
            }
            Rule::expects => {
                for expect in inner.into_inner() {
                    debug_assert_eq!(expect.as_rule(), Rule::expect);
                    let expr = expect.into_inner().next().unwrap();
                    assertions.push(parse_assertion(expr)?);
                }
            }
            _ => {}
        }
    }

    let test_case = TestCase {
        name,
        request: request.context("Test block missing HTTP request")?,
        response: None,
        assertions: assertions,
        result: None,
    };

    Ok(test_case)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn parse_should_succeed_with_valid_input() {
        let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
        let file_path = examples_dir.join("test1.ax");

        let file_content =
            fs::read_to_string(file_path).expect("Should have been able to read the file");

        let result = AxParser::parse_file(&file_content);

        assert!(result.is_ok());
        let test_cases = result.unwrap();
        assert_eq!(test_cases.len(), 1);

        let test_case = &test_cases[0];
        assert_eq!(test_case.name, Some("POST create a resource".to_string()));
        assert_eq!(test_case.request.method, "POST");
    }

    #[test]
    fn parse_should_handle_multiple_tests() {
        let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
        let file_path = examples_dir.join("test2.ax");

        let file_content =
            fs::read_to_string(file_path).expect("Should have been able to read the file");

        let result = AxParser::parse_file(&file_content);

        assert!(result.is_ok());
        let test_cases = result.unwrap();
        assert_eq!(test_cases.len(), 6);
    }

    #[test]
    fn test_parse_http_request() {
        let input = r#"POST https://httpbin.org/post
Content-Type: application/json

BODY
{"name": "test"}
BODYEND"#;
        let mut pairs = AxParser::parse(Rule::request, input).unwrap();
        let request_pair = pairs.next().unwrap();
        let http_request = parse_http_request(request_pair).unwrap();
        assert_eq!(http_request.method, "POST");
        assert_eq!(http_request.url.as_str(), "https://httpbin.org/post");
        assert_eq!(
            http_request.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        match http_request.body {
            Some(Body::Text(text)) => assert_eq!(text.trim(), r#"{"name": "test"}"#),
            _ => panic!("Expected text body"),
        }
    }

    #[test]
    fn test_parse_assertion_binary() {
        let input = "status == 200";
        let mut pairs = AxParser::parse(Rule::expect_expr, input).unwrap();
        let expr_pair = pairs.next().unwrap();
        let assertion = parse_assertion(expr_pair).unwrap();
        match assertion {
            Assertion::Binary { path, op, value } => {
                assert_eq!(path, "status");
                assert_eq!(op, Operator::Eq);
                assert_eq!(value, Value::Number(200));
            }
            _ => panic!("Expected binary assertion"),
        }
    }

    #[test]
    fn test_parse_binary_op() {
        let input = "status == 200";
        let mut pairs = AxParser::parse(Rule::binary_op, input).unwrap();
        let binary_pair = pairs.next().unwrap();
        let assertion = parse_binary_op(binary_pair).unwrap();
        match assertion {
            Assertion::Binary { path, op, value } => {
                assert_eq!(path, "status");
                assert_eq!(op, Operator::Eq);
                assert_eq!(value, Value::Number(200));
            }
            _ => panic!("Expected binary assertion"),
        }
    }

    #[test]
    fn test_parse_operator() {
        let input = "==";
        let mut pairs = AxParser::parse(Rule::operator, input).unwrap();
        let op_pair = pairs.next().unwrap();
        let operator = parse_operator(op_pair).unwrap();
        assert_eq!(operator, Operator::Eq);
    }

    #[test]
    fn test_parse_in_op() {
        let input = "status IN [200, 201]";
        let mut pairs = AxParser::parse(Rule::in_op, input).unwrap();
        let in_pair = pairs.next().unwrap();
        let assertion = parse_in_op(in_pair).unwrap();
        match assertion {
            Assertion::In { path, values } => {
                assert_eq!(path, "status");
                assert_eq!(values, vec![Value::Number(200), Value::Number(201)]);
            }
            _ => panic!("Expected in assertion"),
        }
    }

    #[test]
    fn test_parse_between_op() {
        let input = "body.age BETWEEN 18 AND 65";
        let mut pairs = AxParser::parse(Rule::between_op, input).unwrap();
        let between_pair = pairs.next().unwrap();
        let assertion = parse_between_op(between_pair).unwrap();
        match assertion {
            Assertion::Between { path, min, max } => {
                assert_eq!(path, "body.age");
                assert_eq!(min, Value::Number(18));
                assert_eq!(max, Value::Number(65));
            }
            _ => panic!("Expected between assertion"),
        }
    }

    #[test]
    fn test_parse_exists_op() {
        let input = "body.email EXISTS";
        let mut pairs = AxParser::parse(Rule::exists_op, input).unwrap();
        let exists_pair = pairs.next().unwrap();
        let assertion = parse_exists_op(exists_pair).unwrap();
        match assertion {
            Assertion::Exists { path } => {
                assert_eq!(path, "body.email");
            }
            _ => panic!("Expected exists assertion"),
        }
    }

    #[test]
    fn test_parse_unary_path() {
        let input = "body.active";
        let mut pairs = AxParser::parse(Rule::unary_path, input).unwrap();
        let unary_pair = pairs.next().unwrap();
        let assertion = parse_unary_path(unary_pair).unwrap();
        match assertion {
            Assertion::Unary { path } => {
                assert_eq!(path, "body.active");
            }
            _ => panic!("Expected unary assertion"),
        }
    }

    #[test]
    fn test_parse_value_string() {
        let input = "\"hello\"";
        let mut pairs = AxParser::parse(Rule::value, input).unwrap();
        let value_pair = pairs.next().unwrap();
        let value = parse_value(value_pair).unwrap();
        assert_eq!(value, Value::String("hello".to_string()));
    }

    #[test]
    fn test_parse_value_number() {
        let input = "42";
        let mut pairs = AxParser::parse(Rule::value, input).unwrap();
        let value_pair = pairs.next().unwrap();
        let value = parse_value(value_pair).unwrap();
        assert_eq!(value, Value::Number(42));
    }

    #[test]
    fn test_parse_value_boolean() {
        let input = "true";
        let mut pairs = AxParser::parse(Rule::value, input).unwrap();
        let value_pair = pairs.next().unwrap();
        let value = parse_value(value_pair).unwrap();
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn test_parse_test_block() {
        let input = r#"TEST POST create a resource
POST https://httpbin.org/post
Content-Type: application/json

BODY
{"name": "Axotly"}
BODYEND

EXPECT status == 200
END"#;
        let mut pairs = AxParser::parse(Rule::test_block, input).unwrap();
        let block_pair = pairs.next().unwrap();
        let test_case = parse_test_block(block_pair).unwrap();
        assert_eq!(test_case.name, Some("POST create a resource".to_string()));
        assert_eq!(test_case.request.method, "POST");
        assert_eq!(test_case.assertions.len(), 1);
    }
}
