package kiro

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

const (
	// KiroDir is the base directory for all kiro-related files
	KiroDir = ".kiro"
	// SpecDir is the subdirectory for specifications
	SpecDir = "spec"
	// RequirementsFile is the name of the requirements file
	RequirementsFile = "requirements.md"
)

// SpecManager manages the .kiro/spec directory structure
type SpecManager struct {
	baseDir string
}

// NewSpecManager creates a new SpecManager
func NewSpecManager() *SpecManager {
	return &SpecManager{
		baseDir: KiroDir,
	}
}

// NewSpecManagerWithBase creates a new SpecManager with a custom base directory
func NewSpecManagerWithBase(baseDir string) *SpecManager {
	return &SpecManager{
		baseDir: baseDir,
	}
}

// CreateFeatureDir creates the directory structure for a feature
// Returns the full path to the created directory
func (sm *SpecManager) CreateFeatureDir(featureTitle string) (string, error) {
	// Sanitize the feature title for use as a directory name
	dirName := sanitizeDirName(featureTitle)
	if dirName == "" {
		return "", fmt.Errorf("invalid feature title: %s", featureTitle)
	}

	// Create the full path
	featurePath := filepath.Join(sm.baseDir, SpecDir, dirName)

	// Create the directory
	if err := os.MkdirAll(featurePath, 0755); err != nil {
		return "", fmt.Errorf("failed to create feature directory: %w", err)
	}

	return featurePath, nil
}

// GetRequirementsPath returns the full path to the requirements file for a feature
func (sm *SpecManager) GetRequirementsPath(featureTitle string) (string, error) {
	dirName := sanitizeDirName(featureTitle)
	if dirName == "" {
		return "", fmt.Errorf("invalid feature title: %s", featureTitle)
	}

	return filepath.Join(sm.baseDir, SpecDir, dirName, RequirementsFile), nil
}

// SaveRequirements saves the requirements content to the appropriate file
func (sm *SpecManager) SaveRequirements(featureTitle string, content string) error {
	// Ensure the feature directory exists
	featurePath, err := sm.CreateFeatureDir(featureTitle)
	if err != nil {
		return err
	}

	// Write the requirements file
	requirementsPath := filepath.Join(featurePath, RequirementsFile)
	if err := os.WriteFile(requirementsPath, []byte(content), 0644); err != nil {
		return fmt.Errorf("failed to write requirements file: %w", err)
	}

	return nil
}

// FeatureExists checks if a feature directory already exists
func (sm *SpecManager) FeatureExists(featureTitle string) bool {
	dirName := sanitizeDirName(featureTitle)
	if dirName == "" {
		return false
	}

	featurePath := filepath.Join(sm.baseDir, SpecDir, dirName)
	info, err := os.Stat(featurePath)
	return err == nil && info.IsDir()
}

// sanitizeDirName converts a feature title into a safe directory name
func sanitizeDirName(title string) string {
	// Trim whitespace
	title = strings.TrimSpace(title)
	if title == "" {
		return ""
	}

	// Convert to lowercase
	title = strings.ToLower(title)

	// Replace spaces with hyphens
	title = strings.ReplaceAll(title, " ", "-")

	// Remove or replace unsafe characters
	var safe strings.Builder
	for _, r := range title {
		switch {
		case r >= 'a' && r <= 'z':
			safe.WriteRune(r)
		case r >= '0' && r <= '9':
			safe.WriteRune(r)
		case r == '-' || r == '_':
			safe.WriteRune(r)
		default:
			// Replace other characters with hyphen
			if safe.Len() > 0 && safe.String()[safe.Len()-1] != '-' {
				safe.WriteRune('-')
			}
		}
	}

	// Remove trailing hyphens
	result := strings.TrimSuffix(safe.String(), "-")

	// Ensure it doesn't start with a hyphen
	result = strings.TrimPrefix(result, "-")

	return result
}
