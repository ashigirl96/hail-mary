// // Integration test for document generation
// use std::fs;
// use std::process::Command;
// use tempfile::tempdir;
//
// #[test]
// fn test_document_generation_command() {
//     // Create a temporary directory
//     let temp_dir = tempdir().unwrap();
//     let output_dir = temp_dir.path().join("docs");
//
//     // Run document generation command (it should handle empty database gracefully)
//     let output = Command::new("cargo")
//         .args(&[
//             "run",
//             "--",
//             "memory",
//             "document",
//             "--output",
//             output_dir.to_str().unwrap(),
//         ])
//         .output()
//         .expect("Failed to execute command");
//
//     // Should succeed even with no database
//     assert!(output.status.success());
//
//     // Check that files were created
//     assert!(output_dir.join("tech.md").exists());
//     assert!(output_dir.join("project-tech.md").exists());
//     assert!(output_dir.join("domain.md").exists());
//
//     // Check content of tech.md (should have empty message)
//     let tech_content = fs::read_to_string(output_dir.join("tech.md")).unwrap();
//     assert!(tech_content.contains("Technical Knowledge"));
//     assert!(tech_content.contains("No memories recorded yet"));
// }
