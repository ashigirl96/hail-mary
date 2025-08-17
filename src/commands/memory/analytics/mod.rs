//! Analytics and statistics commands for the Memory MCP system
//!
//! This module provides comprehensive analytics capabilities including:
//! - Summary dashboard and overview statistics
//! - Memory type and confidence analysis
//! - Usage patterns and access analytics
//! - Content analysis and text metrics
//! - Performance and optimization insights
//! - Database health and data quality metrics
//! - Temporal trends and historical analysis

pub mod content;
pub mod health;
pub mod memory;
pub mod performance;
pub mod summary;
pub mod trends;
pub mod usage;

use crate::memory::repository::SqliteMemoryRepository;
use crate::utils::error::Result;
use clap::Args;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

/// Analytics command for comprehensive memory database analysis
#[derive(Args)]
pub struct AnalyticsCommand {
    /// Path to the database file (defaults to ~/.local/share/hail-mary/memory.db)
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,

    /// Output format for analytics data
    #[arg(long, value_enum, default_value = "table")]
    pub format: OutputFormat,

    /// Export analytics data to file
    #[arg(long, value_name = "PATH")]
    pub export: Option<PathBuf>,

    /// Date range filter (start date in YYYY-MM-DD format)
    #[arg(long, value_name = "DATE")]
    pub from_date: Option<String>,

    /// Date range filter (end date in YYYY-MM-DD format)
    #[arg(long, value_name = "DATE")]
    pub to_date: Option<String>,

    /// Include deleted memories in analysis
    #[arg(long)]
    pub include_deleted: bool,

    /// Enable verbose output with detailed metrics
    #[arg(long, short)]
    pub verbose: bool,

    /// Suppress all output except results
    #[arg(long)]
    pub quiet: bool,

    /// Analytics subcommand
    #[command(subcommand)]
    pub command: AnalyticsSubcommand,
}

/// Available analytics subcommands
#[derive(clap::Subcommand)]
pub enum AnalyticsSubcommand {
    /// Overall database summary and key metrics dashboard
    Summary(summary::SummaryCommand),

    /// Memory type distribution and confidence analysis
    Memory(memory::MemoryCommand),

    /// Usage patterns and access analytics
    Usage(usage::UsageCommand),

    /// Content analysis and text metrics
    Content(content::ContentCommand),

    /// Performance metrics and optimization insights
    Performance(performance::PerformanceCommand),

    /// Database health and data quality metrics
    Health(health::HealthCommand),

    /// Temporal trends and historical analysis
    Trends(trends::TrendsCommand),
}

/// Output format options for analytics data
#[derive(Clone, Debug, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable table format
    Table,
    /// JSON format for programmatic use
    Json,
    /// CSV format for spreadsheet import
    Csv,
    /// Markdown format for documentation
    Markdown,
}

/// Common analytics data structure for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsMetric {
    pub name: String,
    pub value: MetricValue,
    pub description: Option<String>,
    pub unit: Option<String>,
}

/// Metric value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetricValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Array(Vec<String>),
}

/// Time range for analytics filtering
#[derive(Debug, Clone)]
pub struct TimeRange {
    #[allow(dead_code)] // Reserved for future time-based filtering functionality
    pub start: Option<i64>,
    #[allow(dead_code)] // Reserved for future time-based filtering functionality
    pub end: Option<i64>,
}

/// Analytics context for shared state across analytics commands
#[derive(Debug)]
pub struct AnalyticsContext {
    pub repository: SqliteMemoryRepository,
    #[allow(dead_code)] // Reserved for future time-based analytics filtering
    pub time_range: TimeRange,
    pub include_deleted: bool,
    #[allow(dead_code)] // Reserved for future enhanced verbose reporting
    pub verbose: bool,
    pub quiet: bool,
}

impl AnalyticsContext {
    /// Create a new analytics context
    pub fn new(
        repository: SqliteMemoryRepository,
        time_range: TimeRange,
        include_deleted: bool,
        verbose: bool,
        quiet: bool,
    ) -> Self {
        Self {
            repository,
            time_range,
            include_deleted,
            verbose,
            quiet,
        }
    }
}

/// Analytics formatter trait for consistent output formatting
pub trait AnalyticsFormatter {
    fn format_metrics(&self, metrics: &[AnalyticsMetric], format: &OutputFormat) -> Result<String>;
    #[allow(dead_code)] // Reserved for future table formatting functionality
    fn format_table(&self, headers: &[String], rows: &[Vec<String>]) -> String;
    fn format_json<T: Serialize + ?Sized>(&self, data: &T) -> Result<String>;
    fn format_csv(&self, headers: &[String], rows: &[Vec<String>]) -> String;
}

/// Default analytics formatter implementation
pub struct DefaultAnalyticsFormatter;

impl AnalyticsFormatter for DefaultAnalyticsFormatter {
    fn format_metrics(&self, metrics: &[AnalyticsMetric], format: &OutputFormat) -> Result<String> {
        match format {
            OutputFormat::Table => {
                let mut result = String::new();
                for metric in metrics {
                    result.push_str(&format!("{:<30} {}\n", metric.name, metric.value));
                }
                Ok(result)
            }
            OutputFormat::Json => self.format_json(metrics),
            OutputFormat::Csv => {
                let headers = vec![
                    "Metric".to_string(),
                    "Value".to_string(),
                    "Unit".to_string(),
                ];
                let rows: Vec<Vec<String>> = metrics
                    .iter()
                    .map(|m| {
                        vec![
                            m.name.clone(),
                            m.value.to_string(),
                            m.unit.as_ref().unwrap_or(&String::new()).clone(),
                        ]
                    })
                    .collect();
                Ok(self.format_csv(&headers, &rows))
            }
            OutputFormat::Markdown => {
                let mut result =
                    String::from("| Metric | Value | Unit |\n|--------|-------|------|\n");
                for metric in metrics {
                    result.push_str(&format!(
                        "| {} | {} | {} |\n",
                        metric.name,
                        metric.value,
                        metric.unit.as_ref().unwrap_or(&String::new())
                    ));
                }
                Ok(result)
            }
        }
    }

    fn format_table(&self, headers: &[String], rows: &[Vec<String>]) -> String {
        let mut result = String::new();

        // Calculate column widths
        let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        // Format header
        for (i, header) in headers.iter().enumerate() {
            result.push_str(&format!("{:<width$}", header, width = widths[i] + 2));
        }
        result.push('\n');

        // Format separator
        for width in &widths {
            result.push_str(&"-".repeat(width + 2));
        }
        result.push('\n');

        // Format rows
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    result.push_str(&format!("{:<width$}", cell, width = widths[i] + 2));
                }
            }
            result.push('\n');
        }

        result
    }

    fn format_json<T: Serialize + ?Sized>(&self, data: &T) -> Result<String> {
        Ok(serde_json::to_string_pretty(data)?)
    }

    fn format_csv(&self, headers: &[String], rows: &[Vec<String>]) -> String {
        let mut result = String::new();

        // Add headers
        result.push_str(&headers.join(","));
        result.push('\n');

        // Add rows
        for row in rows {
            result.push_str(&row.join(","));
            result.push('\n');
        }

        result
    }
}

impl std::fmt::Display for MetricValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricValue::Integer(i) => write!(f, "{}", i),
            MetricValue::Float(fl) => write!(f, "{:.2}", fl),
            MetricValue::String(s) => write!(f, "{}", s),
            MetricValue::Boolean(b) => write!(f, "{}", b),
            MetricValue::Array(arr) => write!(f, "{}", arr.join(", ")),
        }
    }
}

impl AnalyticsCommand {
    /// Execute the analytics command
    pub fn execute(self) -> Result<()> {
        use crate::mcp::server::get_default_db_path;

        // Determine database path
        let db_path = self
            .db_path
            .clone()
            .unwrap_or_else(|| get_default_db_path().expect("Failed to get default database path"));

        // Check if database exists
        if !db_path.exists() {
            eprintln!("Error: Database not found at {:?}", db_path);
            eprintln!("Please run 'hail-mary memory serve' first to create the database.");
            return Ok(());
        }

        let repository = SqliteMemoryRepository::new(&db_path)?;

        // Parse time range
        let time_range = TimeRange {
            start: self.from_date.as_ref().and_then(|d| parse_date(d).ok()),
            end: self.to_date.as_ref().and_then(|d| parse_date(d).ok()),
        };

        // Create analytics context
        let context = AnalyticsContext::new(
            repository,
            time_range,
            self.include_deleted,
            self.verbose,
            self.quiet,
        );

        if self.verbose && !self.quiet {
            println!(
                "📊 Memory Analytics - {}",
                get_command_description(&self.command)
            );
            println!("Database: {:?}", db_path);
            if let Some(ref from) = self.from_date {
                println!("From: {}", from);
            }
            if let Some(ref to) = self.to_date {
                println!("To: {}", to);
            }
            println!("Include deleted: {}", self.include_deleted);
            println!();
        }

        let start_time = Instant::now();

        // Execute the specific analytics command
        let result = match self.command {
            AnalyticsSubcommand::Summary(cmd) => cmd.execute(&context),
            AnalyticsSubcommand::Memory(cmd) => cmd.execute(&context),
            AnalyticsSubcommand::Usage(cmd) => cmd.execute(&context),
            AnalyticsSubcommand::Content(cmd) => cmd.execute(&context),
            AnalyticsSubcommand::Performance(cmd) => cmd.execute(&context),
            AnalyticsSubcommand::Health(cmd) => cmd.execute(&context),
            AnalyticsSubcommand::Trends(cmd) => cmd.execute(&context),
        }?;

        let elapsed = start_time.elapsed();

        // Format and output results
        let formatter = DefaultAnalyticsFormatter;
        let output = formatter.format_metrics(&result, &self.format)?;

        if !self.quiet {
            println!("{}", output);

            if self.verbose {
                println!("📈 Analysis completed in {:?}", elapsed);
            }
        }

        // Export to file if requested
        if let Some(export_path) = self.export {
            std::fs::write(&export_path, &output)?;
            if !self.quiet {
                println!("📄 Results exported to: {:?}", export_path);
            }
        }

        Ok(())
    }
}

/// Parse date string in YYYY-MM-DD format to Unix timestamp
fn parse_date(date_str: &str) -> Result<i64> {
    use chrono::NaiveDate;

    let naive_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|e| {
        crate::utils::error::HailMaryError::General(anyhow::anyhow!("Invalid date format: {}", e))
    })?;

    let datetime = naive_date.and_hms_opt(0, 0, 0).ok_or_else(|| {
        crate::utils::error::HailMaryError::General(anyhow::anyhow!("Invalid time"))
    })?;

    Ok(datetime.and_utc().timestamp())
}

/// Get description for analytics command
fn get_command_description(command: &AnalyticsSubcommand) -> &'static str {
    match command {
        AnalyticsSubcommand::Summary(_) => "Database Summary Dashboard",
        AnalyticsSubcommand::Memory(_) => "Memory Analysis & Distribution",
        AnalyticsSubcommand::Usage(_) => "Usage Patterns & Access Analytics",
        AnalyticsSubcommand::Content(_) => "Content Analysis & Text Metrics",
        AnalyticsSubcommand::Performance(_) => "Performance & Optimization Insights",
        AnalyticsSubcommand::Health(_) => "Database Health & Data Quality",
        AnalyticsSubcommand::Trends(_) => "Temporal Trends & Historical Analysis",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_value_display() {
        assert_eq!(MetricValue::Integer(42).to_string(), "42");
        assert_eq!(MetricValue::Float(std::f64::consts::PI).to_string(), "3.14");
        assert_eq!(MetricValue::String("test".to_string()).to_string(), "test");
        assert_eq!(MetricValue::Boolean(true).to_string(), "true");
        assert_eq!(
            MetricValue::Array(vec!["a".to_string(), "b".to_string()]).to_string(),
            "a, b"
        );
    }

    #[test]
    fn test_analytics_formatter() {
        let formatter = DefaultAnalyticsFormatter;
        let metrics = vec![AnalyticsMetric {
            name: "Total Memories".to_string(),
            value: MetricValue::Integer(100),
            description: None,
            unit: Some("count".to_string()),
        }];

        let table_output = formatter
            .format_metrics(&metrics, &OutputFormat::Table)
            .unwrap();
        assert!(table_output.contains("Total Memories"));
        assert!(table_output.contains("100"));
    }

    #[test]
    fn test_parse_date() {
        assert!(parse_date("2024-01-01").is_ok());
        assert!(parse_date("invalid-date").is_err());
    }
}
