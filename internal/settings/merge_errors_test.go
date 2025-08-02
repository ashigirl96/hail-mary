package settings

import (
	"os"
	"path/filepath"
	"testing"
)

// TestSaveToFileErrorCases tests error paths in SaveToFile
func TestSaveToFileErrorCases(t *testing.T) {
	s := &ClaudeSettings{
		Extra: map[string]interface{}{
			"test": "value",
		},
	}

	// Test case 1: Directory creation failure
	// Create a file where the directory should be
	tempDir := t.TempDir()
	blockingFile := filepath.Join(tempDir, "blocked", "path")

	// Create the parent directory
	os.MkdirAll(filepath.Dir(blockingFile), 0755)

	// Create a file where the subdirectory should be
	if err := os.WriteFile(blockingFile, []byte("block"), 0644); err != nil {
		t.Fatalf("Failed to create blocking file: %v", err)
	}

	// Try to save to a path under the blocking file
	targetPath := filepath.Join(blockingFile, "settings.json")
	err := s.SaveToFile(targetPath)
	if err == nil {
		t.Error("SaveToFile should fail when directory cannot be created")
	}

	// Test case 2: File write permission error
	// Create a directory with no write permission
	readOnlyDir := filepath.Join(tempDir, "readonly")
	os.MkdirAll(readOnlyDir, 0755)

	// Make it read-only
	os.Chmod(readOnlyDir, 0555)
	defer os.Chmod(readOnlyDir, 0755) // Restore for cleanup

	targetPath2 := filepath.Join(readOnlyDir, "settings.json")
	err = s.SaveToFile(targetPath2)
	if err == nil {
		t.Error("SaveToFile should fail when file cannot be written")
	}
}

// TestUnmarshalJSONErrorCases tests additional error paths in UnmarshalJSON
func TestUnmarshalJSONErrorCases(t *testing.T) {
	tests := []struct {
		name    string
		data    []byte
		wantErr bool
	}{
		{
			name: "hooks unmarshal error",
			// This JSON has valid structure but the hooks field contains
			// something that can't be unmarshaled into map[string][]HookMatcher
			data:    []byte(`{"hooks": "not-an-object"}`),
			wantErr: true, // This fails because string can't be unmarshaled into map[string][]HookMatcher
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var s ClaudeSettings
			err := s.UnmarshalJSON(tt.data)
			if (err != nil) != tt.wantErr {
				t.Errorf("UnmarshalJSON() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}

	// The error paths in UnmarshalJSON for unmarshaling hooks are effectively
	// unreachable because map[string]interface{} can hold any JSON value
	t.Log("Note: Some error paths in UnmarshalJSON are effectively unreachable with map[string]interface{}")
}

// TestLoadSettingsReadError tests read error in LoadSettings
func TestLoadSettingsReadError(t *testing.T) {
	// This is covered by the unreadable file test in the main test file
	// Additional platform-specific tests would require OS-specific code
	t.Skip("File read errors are tested in main test file")
}
