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
