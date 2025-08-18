use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::{env, fs};
use tempfile::TempDir;
use tokio::process::Command as AsyncCommand;
use tokio::time::{timeout, Duration};

/// Test utilities for E2E testing of hail-mary commands
pub struct E2ETestEnv {
    pub temp_dir: TempDir,
    pub binary_path: PathBuf,
    pub working_dir: PathBuf,
}

impl E2ETestEnv {
    /// Create a new E2E test environment
    pub fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
        
        // Get the binary path (built by cargo)
        let project_root = env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        let binary_path = project_root.join("target/debug/hail-mary");
        
        let working_dir = temp_dir.path().to_path_buf();
        
        Ok(Self {
            temp_dir,
            binary_path,
            working_dir,
        })
    }
    
    /// Execute a hail-mary command and return the output
    pub fn run_command(&self, args: &[&str]) -> Result<CommandResult> {
        let output = Command::new(&self.binary_path)
            .args(args)
            .current_dir(&self.working_dir)
            .output()
            .context("Failed to execute command")?;
        
        Ok(CommandResult::from_output(output))
    }
    
    /// Execute a hail-mary command asynchronously with timeout
    pub async fn run_command_async(&self, args: &[&str], timeout_secs: u64) -> Result<CommandResult> {
        let mut cmd = AsyncCommand::new(&self.binary_path);
        cmd.args(args).current_dir(&self.working_dir);
        
        let output = timeout(Duration::from_secs(timeout_secs), cmd.output())
            .await
            .context("Command timed out")?
            .context("Failed to execute async command")?;
        
        Ok(CommandResult::from_output(output))
    }
    
    /// Initialize .kiro directory structure
    pub fn init_project(&self) -> Result<CommandResult> {
        self.run_command(&["init", "--force"])
    }
    
    /// Load fixture data from YAML file
    pub fn load_fixture_memories(&self, fixture_name: &str) -> Result<Vec<FixtureMemory>> {
        let project_root = env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        let fixture_path = project_root.join("tests/fixtures").join(fixture_name);
        
        let content = fs::read_to_string(&fixture_path)
            .with_context(|| format!("Failed to read fixture file: {}", fixture_path.display()))?;
        
        let fixture_data: FixtureData = serde_yaml::from_str(&content)
            .context("Failed to parse fixture YAML")?;
        
        Ok(fixture_data.memories)
    }
    
    /// Import fixture memories using the remember tool (via MCP simulation)
    pub async fn import_fixture_memories(&self, fixture_name: &str) -> Result<()> {
        let memories = self.load_fixture_memories(fixture_name)?;
        
        // For E2E testing, we'll use direct repository calls rather than MCP
        // This simulates what the MCP server would do
        self.import_memories_directly(memories).await
    }
    
    /// Direct import of memories to repository (simulates MCP remember calls)
    async fn import_memories_directly(&self, memories: Vec<FixtureMemory>) -> Result<()> {
        use hail_mary::models::kiro::KiroConfig;
        use hail_mary::models::memory::{Memory, MemoryType};
        use hail_mary::repositories::memory::{MemoryRepository, SqliteMemoryRepository};
        
        // Load config from the test environment
        let original_dir = env::current_dir()?;
        env::set_current_dir(&self.working_dir)?;
        
        let config = KiroConfig::load()
            .context("Failed to load config for fixture import")?;
        
        let mut repository = SqliteMemoryRepository::new(&config)
            .context("Failed to create repository for fixture import")?;
        
        // Convert fixture memories to domain memories
        let domain_memories: Vec<Memory> = memories
            .into_iter()
            .map(|fixture| {
                let memory_type: MemoryType = fixture.r#type.parse()
                    .with_context(|| format!("Invalid memory type: {}", fixture.r#type))?;
                
                let memory = Memory::new(memory_type, fixture.title, fixture.content)
                    .with_tags(fixture.tags.unwrap_or_default())
                    .with_confidence(fixture.confidence.unwrap_or(1.0));
                
                Ok(memory)
            })
            .collect::<Result<Vec<_>>>()?;
        
        // Save all memories
        repository.save_batch(&domain_memories)
            .context("Failed to save fixture memories")?;
        
        // Restore original directory
        env::set_current_dir(original_dir)?;
        
        Ok(())
    }
    
    /// Check if .kiro directory structure exists
    pub fn kiro_structure_exists(&self) -> bool {
        let kiro_dir = self.working_dir.join(".kiro");
        let config_file = kiro_dir.join("config.toml");
        let memory_dir = kiro_dir.join("memory");
        
        kiro_dir.exists() && config_file.exists() && memory_dir.exists()
    }
    
    /// Read generated documentation files
    pub fn read_generated_docs(&self, memory_type: &str) -> Result<String> {
        let doc_path = self.working_dir.join(".kiro/memory").join(format!("{}.md", memory_type));
        fs::read_to_string(&doc_path)
            .with_context(|| format!("Failed to read documentation file: {}", doc_path.display()))
    }
    
    /// Validate that database file exists and is accessible
    pub fn validate_database(&self) -> Result<bool> {
        let db_path = self.working_dir.join(".kiro/memory/db.sqlite3");
        Ok(db_path.exists() && db_path.is_file())
    }
    
    /// Get database file size in bytes
    pub fn database_size(&self) -> Result<u64> {
        let db_path = self.working_dir.join(".kiro/memory/db.sqlite3");
        let metadata = fs::metadata(&db_path)
            .context("Failed to get database file metadata")?;
        Ok(metadata.len())
    }
    
    /// Count memories in database using direct SQL query
    pub fn count_memories(&self) -> Result<usize> {
        use hail_mary::models::kiro::KiroConfig;
        use hail_mary::repositories::memory::{MemoryRepository, SqliteMemoryRepository};
        
        let original_dir = env::current_dir()?;
        env::set_current_dir(&self.working_dir)?;
        
        let config = KiroConfig::load()?;
        let repository = SqliteMemoryRepository::new(&config)?;
        let memories = repository.find_all()?;
        
        env::set_current_dir(original_dir)?;
        Ok(memories.len())
    }
}

/// Result of executing a command
#[derive(Debug)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
}

impl CommandResult {
    fn from_output(output: Output) -> Self {
        Self {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            success: output.status.success(),
        }
    }
    
    /// Assert that the command succeeded
    pub fn assert_success(&self) -> Result<&Self> {
        if !self.success {
            anyhow::bail!(
                "Command failed with exit code {}\nSTDOUT: {}\nSTDERR: {}",
                self.exit_code,
                self.stdout,
                self.stderr
            );
        }
        Ok(self)
    }
    
    /// Assert that the command failed
    pub fn assert_failure(&self) -> Result<&Self> {
        if self.success {
            anyhow::bail!(
                "Expected command to fail but it succeeded\nSTDOUT: {}\nSTDERR: {}",
                self.stdout,
                self.stderr
            );
        }
        Ok(self)
    }
    
    /// Assert that stdout contains the given text
    pub fn assert_stdout_contains(&self, text: &str) -> Result<&Self> {
        if !self.stdout.contains(text) {
            anyhow::bail!(
                "Expected stdout to contain '{}'\nActual stdout: {}",
                text,
                self.stdout
            );
        }
        Ok(self)
    }
    
    /// Assert that stderr contains the given text
    pub fn assert_stderr_contains(&self, text: &str) -> Result<&Self> {
        if !self.stderr.contains(text) {
            anyhow::bail!(
                "Expected stderr to contain '{}'\nActual stderr: {}",
                text,
                self.stderr
            );
        }
        Ok(self)
    }
}

/// Fixture memory structure for YAML parsing
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FixtureMemory {
    pub r#type: String,
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
    pub confidence: Option<f32>,
}

/// Root structure for fixture YAML files
#[derive(Debug, Serialize, Deserialize)]
pub struct FixtureData {
    pub memories: Vec<FixtureMemory>,
}

/// Mock MCP client for testing MCP server functionality
pub struct MockMcpClient {
    pub server_process: Option<tokio::process::Child>,
}

impl MockMcpClient {
    /// Start MCP server process
    pub async fn start_server(env: &E2ETestEnv) -> Result<Self> {
        let mut cmd = AsyncCommand::new(&env.binary_path);
        cmd.args(&["memory", "serve"])
            .current_dir(&env.working_dir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        
        let child = cmd.spawn().context("Failed to start MCP server")?;
        
        Ok(Self {
            server_process: Some(child),
        })
    }
    
    /// Send MCP request (simplified for testing)
    pub async fn send_request(&mut self, _method: &str, _params: serde_json::Value) -> Result<serde_json::Value> {
        // For E2E testing, we'll simulate MCP interactions
        // In a real implementation, this would send JSON-RPC requests
        // For now, we'll use direct repository calls as a simulation
        Ok(serde_json::json!({
            "result": "success",
            "message": "Mock MCP response"
        }))
    }
    
    /// Stop the MCP server
    pub async fn stop_server(&mut self) -> Result<()> {
        if let Some(mut process) = self.server_process.take() {
            process.kill().await.context("Failed to kill MCP server process")?;
            process.wait().await.context("Failed to wait for MCP server to exit")?;
        }
        Ok(())
    }
}

impl Drop for MockMcpClient {
    fn drop(&mut self) {
        if let Some(mut process) = self.server_process.take() {
            // Attempt to kill the process in blocking mode
            let _ = std::process::Command::new("kill")
                .args(&["-9", &process.id().unwrap().to_string()])
                .output();
        }
    }
}

/// Performance measurement utilities
pub struct PerformanceMeasurement {
    start_time: std::time::Instant,
}

impl PerformanceMeasurement {
    /// Start timing an operation
    pub fn start() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }
    
    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u128 {
        self.start_time.elapsed().as_millis()
    }
    
    /// Assert that operation completed within time limit
    pub fn assert_under_ms(&self, limit_ms: u128) -> Result<()> {
        let elapsed = self.elapsed_ms();
        if elapsed > limit_ms {
            anyhow::bail!(
                "Operation took {}ms, expected under {}ms",
                elapsed,
                limit_ms
            );
        }
        Ok(())
    }
}

/// Validation helpers for E2E tests
pub struct E2EValidation;

impl E2EValidation {
    /// Validate generated markdown content structure
    pub fn validate_markdown_structure(content: &str) -> Result<()> {
        // Check for basic markdown structure
        if !content.contains("##") {
            anyhow::bail!("Generated markdown should contain headers (##)");
        }
        
        if !content.contains("*Tags:") {
            anyhow::bail!("Generated markdown should contain tag information");
        }
        
        if !content.contains("*References:") {
            anyhow::bail!("Generated markdown should contain reference information");
        }
        
        Ok(())
    }
    
    /// Validate Japanese text search quality
    pub fn validate_japanese_search_quality(
        query: &str,
        results: &[String],
        expected_count: usize,
    ) -> Result<()> {
        if results.len() < expected_count {
            anyhow::bail!(
                "Japanese search for '{}' returned {} results, expected at least {}",
                query,
                results.len(),
                expected_count
            );
        }
        
        // Check that results actually contain the search term
        let relevant_results = results
            .iter()
            .filter(|result| result.to_lowercase().contains(&query.to_lowercase()))
            .count();
        
        let precision = relevant_results as f64 / results.len() as f64;
        if precision < 0.8 {
            anyhow::bail!(
                "Search precision for '{}' is {:.2}, expected >= 0.8",
                query,
                precision
            );
        }
        
        Ok(())
    }
    
    /// Validate database file size constraints
    pub fn validate_database_size(size_bytes: u64, max_size_mb: u64) -> Result<()> {
        let size_mb = size_bytes / (1024 * 1024);
        if size_mb > max_size_mb {
            anyhow::bail!(
                "Database size {}MB exceeds maximum {}MB",
                size_mb,
                max_size_mb
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e2e_env_creation() {
        let env = E2ETestEnv::new().unwrap();
        assert!(env.temp_dir.path().exists());
        assert!(env.binary_path.ends_with("hail-mary"));
    }

    #[test]
    fn test_command_result_assertions() {
        let success_result = CommandResult {
            stdout: "Success output".to_string(),
            stderr: "".to_string(),
            exit_code: 0,
            success: true,
        };
        
        success_result.assert_success().unwrap();
        success_result.assert_stdout_contains("Success").unwrap();
        
        let failure_result = CommandResult {
            stdout: "".to_string(),
            stderr: "Error occurred".to_string(),
            exit_code: 1,
            success: false,
        };
        
        failure_result.assert_failure().unwrap();
        failure_result.assert_stderr_contains("Error").unwrap();
    }

    #[test]
    fn test_performance_measurement() {
        let measurement = PerformanceMeasurement::start();
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let elapsed = measurement.elapsed_ms();
        assert!(elapsed >= 10);
        assert!(elapsed < 100); // Should be much less than 100ms
    }
}