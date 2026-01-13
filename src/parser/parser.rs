use pest_derive::Parser;
use pest::Parser;
use pest::iterators::Pair;
use std::collections::HashMap;
use url::Url;
use anyhow::{Result, Context};
use anyhow::bail;

use crate::domain::{TestCase, Assertion};
use crate::domain::assertion::{Operator, Value};
use crate::domain::http_request::{HttpRequest, Body};

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
                url = Some(Url::parse(url_str)
                    .with_context(|| format!("Invalid URL: {}", url_str))?);
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
    let op   = parse_operator(inner.next().unwrap())?;
    let val  = parse_value(inner.next().unwrap())?;

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
        ">"  => Operator::Gt,
        "<"  => Operator::Lt,
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
    let min  = parse_value(inner.next().unwrap())?;
    let max  = parse_value(inner.next().unwrap())?;

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

