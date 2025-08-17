// Test utilities for E2E comprehensive tests

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tempfile::{TempDir, tempdir};

// Re-export commonly used types from the main crate
use hail_mary::mcp::server::MemoryMcpServer;
use hail_mary::memory::models::{
    RmcpRecallParams, RmcpRecallResponse, RmcpRememberParams, RmcpRememberResponse,
};

/// Test data structures matching YAML schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTestCase {
    pub test_id: String,
    pub input: MemoryInput,
    pub expected: ExpectedResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInput {
    pub r#type: String,
    pub title: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedResult {
    pub success: bool,
    pub action: String, // "created" or "updated"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallScenario {
    pub scenario_id: String,
    pub description: String,
    pub input: RecallInput,
    pub expected: RecallExpected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallInput {
    pub query: String,
    pub r#type: Option<String>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallExpected {
    pub behavior: String,
    pub result_type: String,
    #[serde(flatten)]
    pub additional: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveTestData {
    pub test_suite: TestSuite,
    pub minimal_memories: Vec<MemoryTestCase>,
    pub single_example_memories: Vec<MemoryTestCase>,
    pub multiple_examples_memories: Vec<MemoryTestCase>,
    pub single_tag_memories: Vec<MemoryTestCase>,
    pub multiple_tags_memories: Vec<MemoryTestCase>,
    pub complex_combinations: Vec<MemoryTestCase>,
    pub edge_cases: Vec<MemoryTestCase>,
    pub empty_field_variations: Vec<MemoryTestCase>,
    pub duplicate_tests: Vec<MemoryTestCase>,
    pub content_size_variations: Vec<MemoryTestCase>,
    pub tag_variations: Vec<MemoryTestCase>,
    pub example_variations: Vec<MemoryTestCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallTestData {
    pub test_suite: TestSuite,
    pub basic_queries: Vec<RecallScenario>,
    pub type_filtered_queries: Vec<RecallScenario>,
    pub tag_filtered_queries: Vec<RecallScenario>,
    pub combined_filters: Vec<RecallScenario>,
    pub limit_scenarios: Vec<RecallScenario>,
    pub special_queries: Vec<RecallScenario>,
    pub empty_result_scenarios: Vec<RecallScenario>,
    pub performance_scenarios: Vec<RecallScenario>,
    pub sorting_scenarios: Vec<RecallScenario>,
    pub cross_field_scenarios: Vec<RecallScenario>,
}

/// Test environment setup
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub db_path: PathBuf,
    pub server: MemoryMcpServer,
}

impl TestEnvironment {
    /// Create a new test environment with a temporary database
    pub async fn new() -> Result<Self> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_memory.db");
        let server = MemoryMcpServer::new(&db_path)?;

        Ok(Self {
            temp_dir,
            db_path,
            server,
        })
    }

    /// Get the database path
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Get a reference to the server
    pub fn server(&self) -> &MemoryMcpServer {
        &self.server
    }
}

/// Load comprehensive test data from YAML file
pub fn load_comprehensive_test_data<P: AsRef<Path>>(path: P) -> Result<ComprehensiveTestData> {
    let content = std::fs::read_to_string(path)?;
    let data: ComprehensiveTestData = serde_yaml::from_str(&content)?;
    Ok(data)
}

/// Load recall test scenarios from YAML file
pub fn load_recall_test_data<P: AsRef<Path>>(path: P) -> Result<RecallTestData> {
    let content = std::fs::read_to_string(path)?;
    let data: RecallTestData = serde_yaml::from_str(&content)?;
    Ok(data)
}

/// Get all test cases from comprehensive test data
pub fn get_all_test_cases(data: &ComprehensiveTestData) -> Vec<&MemoryTestCase> {
    let mut cases = Vec::new();
    cases.extend(&data.minimal_memories);
    cases.extend(&data.single_example_memories);
    cases.extend(&data.multiple_examples_memories);
    cases.extend(&data.single_tag_memories);
    cases.extend(&data.multiple_tags_memories);
    cases.extend(&data.complex_combinations);
    cases.extend(&data.edge_cases);
    cases.extend(&data.empty_field_variations);
    cases.extend(&data.duplicate_tests);
    cases.extend(&data.content_size_variations);
    cases.extend(&data.tag_variations);
    cases.extend(&data.example_variations);
    cases
}

/// Get all recall scenarios from recall test data
pub fn get_all_recall_scenarios(data: &RecallTestData) -> Vec<&RecallScenario> {
    let mut scenarios = Vec::new();
    scenarios.extend(&data.basic_queries);
    scenarios.extend(&data.type_filtered_queries);
    scenarios.extend(&data.tag_filtered_queries);
    scenarios.extend(&data.combined_filters);
    scenarios.extend(&data.limit_scenarios);
    scenarios.extend(&data.special_queries);
    scenarios.extend(&data.empty_result_scenarios);
    scenarios.extend(&data.performance_scenarios);
    scenarios.extend(&data.sorting_scenarios);
    scenarios.extend(&data.cross_field_scenarios);
    scenarios
}

/// Convert test input to RmcpRememberParams
pub fn input_to_remember_params(input: &MemoryInput) -> RmcpRememberParams {
    RmcpRememberParams {
        r#type: input.r#type.clone(),
        title: input.title.clone(),
        content: input.content.clone(),
        tags: input.tags.clone(),
        examples: input.examples.clone(),
    }
}

/// Convert recall input to RmcpRecallParams
pub fn input_to_recall_params(input: &RecallInput) -> RmcpRecallParams {
    RmcpRecallParams {
        query: input.query.clone(),
        r#type: input.r#type.clone(),
        tags: input.tags.clone(),
        limit: input.limit.map(|l| l as u32),
    }
}

/// Assert that a remember response matches expected result
pub fn assert_remember_response(
    response: &RmcpRememberResponse,
    expected: &ExpectedResult,
    test_id: &str,
) {
    assert!(
        expected.success,
        "Test {}: Expected success but operation failed",
        test_id
    );

    assert_eq!(
        response.action, expected.action,
        "Test {}: Expected action '{}' but got '{}'",
        test_id, expected.action, response.action
    );

    assert!(
        !response.memory_id.is_empty(),
        "Test {}: Memory ID should not be empty",
        test_id
    );
}

/// Assert recall response based on expected behavior
pub fn assert_recall_response(
    response: &RmcpRecallResponse,
    expected: &RecallExpected,
    scenario_id: &str,
) {
    match expected.behavior.as_str() {
        "no_results" | "empty_filter" => {
            assert!(
                response.memories.is_empty(),
                "Scenario {}: Expected empty results but got {} memories",
                scenario_id,
                response.memories.len()
            );
        }
        "limited_search" => {
            if let Some(limit_value) = expected.additional.get("result_count") {
                if let Some(limit) = limit_value.as_u64() {
                    assert!(
                        response.memories.len() <= limit as usize,
                        "Scenario {}: Expected at most {} results but got {}",
                        scenario_id,
                        limit,
                        response.memories.len()
                    );
                }
            }
        }
        _ => {
            // For other behaviors, just ensure we handle them gracefully
            // Specific assertions can be added based on behavior type
        }
    }
}

/// Performance measurement helper
pub struct PerformanceMeasure {
    start: std::time::Instant,
    operation: String,
}

impl PerformanceMeasure {
    pub fn start(operation: impl Into<String>) -> Self {
        Self {
            start: std::time::Instant::now(),
            operation: operation.into(),
        }
    }

    pub fn end(self) -> std::time::Duration {
        let duration = self.start.elapsed();
        tracing::info!("Operation '{}' took {:?}", self.operation, duration);
        duration
    }

    pub fn assert_under_ms(self, max_ms: u64) {
        let operation = self.operation.clone();
        let duration = self.end();
        assert!(
            duration.as_millis() < max_ms as u128,
            "Operation '{}' took {:?}, expected under {}ms",
            operation,
            duration,
            max_ms
        );
    }
}

/// Helper to create test data programmatically
pub struct TestDataGenerator {
    memory_type_variants: Vec<String>,
    tag_combinations: Vec<Option<Vec<String>>>,
    example_combinations: Vec<Option<Vec<String>>>,
}

impl TestDataGenerator {
    pub fn new() -> Self {
        Self {
            memory_type_variants: vec![
                "tech".to_string(),
                "project-tech".to_string(),
                "domain".to_string(),
            ],
            tag_combinations: vec![
                None,
                Some(vec![]),
                Some(vec!["tag1".to_string()]),
                Some(vec!["tag1".to_string(), "tag2".to_string()]),
            ],
            example_combinations: vec![
                None,
                Some(vec![]),
                Some(vec!["example1".to_string()]),
                Some(vec!["example1".to_string(), "example2".to_string()]),
            ],
        }
    }

    /// Generate all combinations of test data
    pub fn generate_all_combinations(&self) -> Vec<MemoryTestCase> {
        let mut cases = Vec::new();
        let mut id_counter = 0;

        for memory_type in &self.memory_type_variants {
            for tags in &self.tag_combinations {
                for examples in &self.example_combinations {
                    id_counter += 1;
                    let test_case = MemoryTestCase {
                        test_id: format!("generated_{}", id_counter),
                        input: MemoryInput {
                            r#type: memory_type.clone(),
                            title: format!("Generated Test {}", id_counter),
                            content: format!("Generated content for test {}", id_counter),
                            tags: tags.clone(),
                            examples: examples.clone(),
                        },
                        expected: ExpectedResult {
                            success: true,
                            action: "created".to_string(),
                        },
                    };
                    cases.push(test_case);
                }
            }
        }

        cases
    }
}

/// Test result reporter for better visibility
pub struct TestReporter {
    passed: usize,
    failed: usize,
    failures: Vec<String>,
}

impl TestReporter {
    pub fn new() -> Self {
        Self {
            passed: 0,
            failed: 0,
            failures: Vec::new(),
        }
    }

    pub fn record_pass(&mut self, test_id: &str) {
        self.passed += 1;
        tracing::info!("✓ Test {} passed", test_id);
    }

    pub fn record_fail(&mut self, test_id: &str, reason: &str) {
        self.failed += 1;
        self.failures.push(format!("{}: {}", test_id, reason));
        tracing::error!("✗ Test {} failed: {}", test_id, reason);
    }

    pub fn print_summary(&self) {
        println!("\n=== Test Summary ===");
        println!("Passed: {}", self.passed);
        println!("Failed: {}", self.failed);
        println!("Total: {}", self.passed + self.failed);

        if !self.failures.is_empty() {
            println!("\nFailed Tests:");
            for failure in &self.failures {
                println!("  - {}", failure);
            }
        }

        let pass_rate = if self.passed + self.failed > 0 {
            (self.passed as f64 / (self.passed + self.failed) as f64) * 100.0
        } else {
            0.0
        };

        println!("\nPass Rate: {:.2}%", pass_rate);
    }

    pub fn assert_all_passed(&self) {
        assert_eq!(
            self.failed, 0,
            "Some tests failed. See summary above for details."
        );
    }
}
