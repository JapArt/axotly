
use owo_colors::OwoColorize;
use reqwest::StatusCode;
use std::time::Duration;

use crate::domain::http_request::HttpResponse;

pub struct ResponseRenderer;

impl ResponseRenderer {
    pub fn print_status(status: StatusCode) {
        let colored = match status.as_u16() {
            200..=299 => status.as_str().green().to_string(),
            300..=399 => status.as_str().cyan().to_string(),
            400..=499 => status.as_str().yellow().to_string(),
            _ => status.as_str().red().to_string(),
        };

        println!("{} {}", "Status:".bold(), colored);
    }

    pub fn print_duration(elapsed: Duration) {
        let ms = elapsed.as_millis();

        let colored = if ms < 500 {
            format!("{ms} ms").green().to_string()
        } else if ms < 2000 {
            format!("{ms} ms").yellow().to_string()
        } else {
            format!("{ms} ms").red().to_string()
        };

        println!("{} {}", "Duration:".bold(), colored);
    }

    pub fn print_method(method: &str) {
        let colored = match method.to_uppercase().as_str() {
            "GET" => method.green().to_string(),
            "POST" => method.yellow().to_string(),
            "PUT" | "PATCH" => method.blue().to_string(),
            "DELETE" => method.red().to_string(),
            _ => method.white().to_string(),
        };

        println!("{} {}", "Method:".bold(), colored);
    }

    pub fn print_url(url: &str) {
        println!("{} {}", "URL:".bold(), url.underline());
    }

    pub fn print_headers(headers: &std::collections::HashMap<String, String>) {
        println!("\n{}", "Headers:".bold().purple().to_string());
        for (key, value) in headers {
            println!(" {}: {}", key.blue().to_string(), value);
        }
    }

    pub fn print_body(body: &str) {
        println!("\n{}", "Body:".bold().purple().to_string());
        println!("{body}");
    }

    pub fn print_response(response: &HttpResponse) {
        let status = StatusCode::from_u16(response.status)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let request = &response.request.clone().unwrap();

        Self::print_method(&request.method);
        Self::print_url(&request.url.to_string());

        Self::print_status(status);
        Self::print_duration(response.duration);
        Self::print_headers(&response.headers);

        if let Some(body) = &response.body {
            Self::print_body(body);
        }
    }
}
