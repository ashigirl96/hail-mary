// E2E Comprehensive Test Suite for Memory MCP Server
//
// This test suite provides comprehensive end-to-end testing for the Memory MCP server,
// covering all possible field combinations, query scenarios, and workflows.

#![cfg(test)]

pub mod integration_tests;
pub mod recall_tests;
pub mod remember_tests;
pub mod utils;

// Run all E2E tests with:
// ```bash
// cargo test --test e2e_comprehensive_main
// ```
//
// Run specific test modules:
// ```bash
// cargo test --test e2e_comprehensive_main remember_tests
// cargo test --test e2e_comprehensive_main recall_tests
// cargo test --test e2e_comprehensive_main integration_tests
// ```
//
// Run with logging:
// ```bash
// RUST_LOG=debug cargo test --test e2e_comprehensive_main -- --nocapture
// ```
