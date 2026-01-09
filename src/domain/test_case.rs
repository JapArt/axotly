use std::time::Duration;
use super::http_request::{HttpRequest, HttpResponse};
use crate::domain::{Assertion, AssertionFailure};

/// Result of executing a test case
#[derive(Debug)]
pub enum TestResult {
    Passed {
        duration: Duration,
    },
    Failed {
        duration: Duration,
        errors: Vec<AssertionFailure>,
    },
}

#[derive(Debug)]
pub struct TestCase {
    pub name: Option<String>,
    pub request: HttpRequest,
    pub response: Option<HttpResponse>,
    pub assertions: Vec<Assertion>,
    pub result: Option<TestResult>,
}

impl TestCase {
    pub async fn run(mut self) -> TestCase {
        let start = std::time::Instant::now();

        let response = match self.request.clone().send().await {
            Ok(res) => res,
            Err(error) => {
                self.result = Some(TestResult::Failed {
                    duration: start.elapsed(),
                    errors: vec![AssertionFailure {
                        path: "request".into(),
                        expected: None,
                        actual: None,
                        message: error.to_string(),
                    }],
                });
                return self;
            }
        };

        self.response = Some(response.clone());

        let mut errors = Vec::new();

        for assertion in &self.assertions {
            if let Err(err) = assertion.check(&response) {
                errors.push(err);
            }
        }

        if errors.is_empty() {
            self.result = Some(TestResult::Passed {
                duration: start.elapsed(),
            });
        } else {
            self.result = Some(TestResult::Failed {
                duration: start.elapsed(),
                errors,
            });
        }

        self
    }
}
