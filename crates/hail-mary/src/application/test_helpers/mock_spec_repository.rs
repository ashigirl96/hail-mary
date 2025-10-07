//! Mock implementation of SpecRepositoryInterface for testing

use crate::application::errors::ApplicationError;
use crate::application::repositories::SpecRepositoryInterface;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Debug, Default)]
pub struct MockSpecRepository {
    specs: RwLock<HashSet<String>>,
    archived_specs: RwLock<HashSet<String>>,
    operations_to_fail: RwLock<HashMap<String, bool>>,
}

impl MockSpecRepository {
    pub fn new() -> Self {
        Self {
            specs: RwLock::new(HashSet::new()),
            archived_specs: RwLock::new(HashSet::new()),
            operations_to_fail: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_specs(specs: Vec<String>) -> Self {
        Self {
            specs: RwLock::new(specs.into_iter().collect()),
            archived_specs: RwLock::new(HashSet::new()),
            operations_to_fail: RwLock::new(HashMap::new()),
        }
    }

    pub fn set_operation_to_fail(&self, operation: &str) {
        self.operations_to_fail
            .write()
            .unwrap()
            .insert(operation.to_string(), true);
    }

    pub fn clear_failure(&self) {
        self.operations_to_fail.write().unwrap().clear();
    }

    fn should_fail(&self, operation: &str) -> bool {
        self.operations_to_fail
            .read()
            .unwrap()
            .get(operation)
            .copied()
            .unwrap_or(false)
    }

    pub fn get_created_specs(&self) -> Vec<String> {
        self.specs.read().unwrap().iter().cloned().collect()
    }
}

impl SpecRepositoryInterface for MockSpecRepository {
    fn create_spec(&self, name: &str) -> Result<(), ApplicationError> {
        if self.should_fail("create_spec") {
            return Err(ApplicationError::SpecCreationError(format!(
                "Mock creation failure for {}",
                name
            )));
        }

        // Validate spec name (kebab-case)
        if !name
            .chars()
            .all(|c| c.is_lowercase() || c == '-' || c.is_numeric())
            || name.starts_with('-')
            || name.ends_with('-')
            || name.contains("--")
        {
            return Err(ApplicationError::InvalidSpecName(name.to_string()));
        }

        let date = chrono::Utc::now().format("%Y-%m-%d");
        let spec_name = format!("{}-{}", date, name);
        self.specs.write().unwrap().insert(spec_name);
        Ok(())
    }

    fn list_spec_directories(&self) -> Result<Vec<(String, bool)>, ApplicationError> {
        if self.should_fail("list_spec_directories") {
            return Err(ApplicationError::FileSystemError(
                "Mock list failure".to_string(),
            ));
        }

        let specs = self
            .specs
            .read()
            .unwrap()
            .iter()
            .map(|spec| (spec.clone(), false))
            .collect();
        Ok(specs)
    }

    fn mark_spec_complete(&self, name: &str) -> Result<(), ApplicationError> {
        if self.should_fail("mark_spec_complete") {
            return Err(ApplicationError::FileSystemError(
                "Mock complete failure".to_string(),
            ));
        }

        let mut specs = self.specs.write().unwrap();
        let mut archived = self.archived_specs.write().unwrap();

        if specs.remove(name) {
            archived.insert(name.to_string());
            Ok(())
        } else {
            Err(ApplicationError::SpecNotFound(name.to_string()))
        }
    }

    fn get_spec_path(&self, name: &str) -> Result<PathBuf, ApplicationError> {
        if self.should_fail("get_spec_path") {
            return Err(ApplicationError::FileSystemError(
                "Mock get path failure".to_string(),
            ));
        }

        let specs = self.specs.read().unwrap();
        if specs.contains(name) {
            Ok(PathBuf::from(format!(".kiro/specs/{}", name)))
        } else {
            Err(ApplicationError::SpecNotFound(name.to_string()))
        }
    }

    fn list_archived_specs(&self) -> Result<Vec<String>, ApplicationError> {
        if self.should_fail("list_archived_specs") {
            return Err(ApplicationError::FileSystemError(
                "Mock list archived failure".to_string(),
            ));
        }

        let archived = self
            .archived_specs
            .read()
            .unwrap()
            .iter()
            .cloned()
            .collect();
        Ok(archived)
    }

    fn is_pbi(&self, _spec_name: &str) -> Result<bool, ApplicationError> {
        // Mock: always return false for simplicity
        Ok(false)
    }

    fn list_sbis(&self, _pbi_name: &str) -> Result<Vec<String>, ApplicationError> {
        // Mock: return empty vec
        Ok(Vec::new())
    }

    fn create_sbi(&self, _pbi_name: &str, _sbi_name: &str) -> Result<(), ApplicationError> {
        // Mock: do nothing
        Ok(())
    }
}
