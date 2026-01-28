//! Asynchronous test executor with bounded concurrency.
//!
//! This module defines the [`Executor`], responsible for running a collection
//! of [`TestCase`] instances concurrently while enforcing a maximum level of
//! parallelism.
//!
//! Concurrency is controlled using a Tokio [`Semaphore`], ensuring that no more
//! than `max_concurrency` test cases are executed at the same time. Each test
//! case is spawned as a Tokio task and acquires a semaphore permit before
//! running.
//!
//! The executor is intentionally stateless. It receives all required input
//! (the test cases and concurrency limit) and returns the executed test cases
//! with their results populated.
//!
//! ## Design goals
//!
//! - **Bounded parallelism**: Prevent resource exhaustion by limiting
//!   simultaneous test execution.
//! - **Task isolation**: Each test runs in its own Tokio task.
//! - **Simplicity**: No internal state or lifecycle management.
//!
//! ## Execution model
//!
//! 1. A semaphore is created with `max_concurrency` permits.
//! 2. Each [`TestCase`] is spawned as an async task.
//! 3. Before running, the task acquires a semaphore permit.
//! 4. The test is executed via [`TestCase::run`].
//! 5. Completed test cases are collected and returned.
//!
//! Failed or panicked tasks are ignored and not included in the results.


use std::sync::Arc;
use tokio::sync::Semaphore;
use crate::domain::TestCase;

pub struct Executor;

impl Executor {
    pub async fn run_tests(test_cases: Vec<TestCase>, max_concurrency: usize) -> Vec<TestCase> {
        let semaphore = Arc::new(Semaphore::new(max_concurrency));
        let mut handles = Vec::new();

        for test_case in test_cases {
            let sem = Arc::clone(&semaphore);
            
            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.expect("Semaphore closed");
                let result = test_case.run().await;
                result
            });
            
            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(test_case) = handle.await {
                results.push(test_case);
            }
        }

        results
    }
}
