use std::collections::HashMap;
use url::Url;
use anyhow::Result;
use reqwest::{Client, Method as ReqwestMethod, Response};

/// HTTP request domain object
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub url: Url,
    pub headers: HashMap<String, String>,
    pub body: Option<Body>,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub request: Option<HttpRequest>,
    pub duration: std::time::Duration,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}


/// Request body
#[derive(Debug, Clone)]
pub enum Body {
    Text(String),
    Json(serde_json::Value),
}

impl HttpRequest {
    pub fn new(method: String, url: Url) -> Self {
        Self {
            method: method.to_uppercase(),
            url,
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn body(mut self, body: Option<Body>) -> Self {
        self.body = body;
        self
    }

    pub async fn call_request(&self) -> Result<Response> {
        let client = Client::new();

        // Method mapping (string → reqwest)
        let method = match self.method.as_str() {
            "GET" => ReqwestMethod::GET,
            "POST" => ReqwestMethod::POST,
            "PUT" => ReqwestMethod::PUT,
            "PATCH" => ReqwestMethod::PATCH,
            "DELETE" => ReqwestMethod::DELETE,
            "HEAD" => ReqwestMethod::HEAD,
            "OPTIONS" => ReqwestMethod::OPTIONS,
            _ => return Err(anyhow::anyhow!("Invalid HTTP method: {}", self.method)),
        };

        let mut req = client.request(method, self.url.as_str());

        // Headers
        for (key, value) in &self.headers {
            req = req.header(key, value);
        }

        // Body
        if let Some(body) = &self.body {
            match body {
                Body::Text(text) => {
                    req = req.body(text.clone());
                }
                Body::Json(value) => {
                    req = req
                        .header("Content-Type", "application/json")
                        .json(value);
                }
            }
        }

        let response = req.send().await?;

        Ok(response)
    }

    pub async fn send(self) -> anyhow::Result<HttpResponse> {
        let start = std::time::Instant::now();
        
        let response = self.call_request().await?;
        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect::<HashMap<String, String>>();
        let body = response.text().await?;

        let duration = start.elapsed();

        Ok(HttpResponse {
            request: Some(self),
            duration,
            status,
            headers,
            body: Some(body),
        })
    }
}
