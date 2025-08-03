package claude

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"
)

// TranscriptEntry represents a single line in a Claude transcript JSONL file
type TranscriptEntry struct {
	ParentUUID            string                 `json:"parentUuid"`
	IsSidechain           bool                   `json:"isSidechain"`
	UserType              string                 `json:"userType"`
	CWD                   string                 `json:"cwd"`
	SessionID             string                 `json:"sessionId"`
	Version               string                 `json:"version"`
	GitBranch             string                 `json:"gitBranch"`
	Type                  string                 `json:"type"`
	Message               map[string]interface{} `json:"message,omitempty"`
	UUID                  string                 `json:"uuid"`
	Timestamp             string                 `json:"timestamp"`
	IsVisibleInTranscript *bool                  `json:"isVisibleInTranscriptOnly,omitempty"`
	Content               string                 `json:"content,omitempty"`
	IsMeta                *bool                  `json:"isMeta,omitempty"`
	ToolUseID             string                 `json:"toolUseID,omitempty"`
	Level                 string                 `json:"level,omitempty"`
	RequestID             string                 `json:"requestId,omitempty"`
	ToolUseResult         map[string]interface{} `json:"toolUseResult,omitempty"`
}

// TruncateTranscript creates a new transcript file truncated at the specified turn number
// It reads the original transcript and copies entries up to and including the specified turn
func TruncateTranscript(originalPath string, turnNumber int) (string, error) {
	// Open the original file
	file, err := os.Open(originalPath)
	if err != nil {
		return "", fmt.Errorf("failed to open transcript: %w", err)
	}
	defer file.Close()

	// Create a temporary file for the truncated transcript
	dir := filepath.Dir(originalPath)
	tempFile, err := os.CreateTemp(dir, "truncated-*.jsonl")
	if err != nil {
		return "", fmt.Errorf("failed to create temp file: %w", err)
	}
	tempPath := tempFile.Name()

	// Create a backup of the original
	backupPath := originalPath + ".backup." + time.Now().Format("20060102-150405")
	if err := copyFile(originalPath, backupPath); err != nil {
		tempFile.Close()
		os.Remove(tempPath)
		return "", fmt.Errorf("failed to create backup: %w", err)
	}

	scanner := bufio.NewScanner(file)
	writer := bufio.NewWriter(tempFile)
	currentTurn := 0
	foundTargetTurn := false
	lastParentUUID := ""

	for scanner.Scan() {
		line := scanner.Text()
		var entry TranscriptEntry

		if err := json.Unmarshal([]byte(line), &entry); err != nil {
			// If we can't parse it, still write it (might be metadata)
			_, _ = writer.WriteString(line + "\n")
			continue
		}

		// Track user messages that aren't hooks
		if entry.Type == "user" && entry.Message != nil {
			if role, ok := entry.Message["role"].(string); ok && role == "user" {
				if content, ok := entry.Message["content"].(string); ok && !strings.Contains(content, "-hook>") {
					currentTurn++
					if currentTurn == turnNumber {
						foundTargetTurn = true
						lastParentUUID = entry.UUID
					}
				} else if contentArray, ok := entry.Message["content"].([]interface{}); ok {
					// Handle content as array
					isHook := false
					for _, item := range contentArray {
						if textItem, ok := item.(map[string]interface{}); ok {
							if text, ok := textItem["text"].(string); ok && strings.Contains(text, "-hook>") {
								isHook = true
								break
							}
						}
					}
					if !isHook {
						currentTurn++
						if currentTurn == turnNumber {
							foundTargetTurn = true
							lastParentUUID = entry.UUID
						}
					}
				}
			}
		}

		// Write the entry
		_, _ = writer.WriteString(line + "\n")

		// If we've found the target turn and this entry's parent is the target turn,
		// we've written the immediate response and should stop
		if foundTargetTurn && entry.ParentUUID == lastParentUUID {
			break
		}
	}

	if err := scanner.Err(); err != nil {
		writer.Flush()
		tempFile.Close()
		os.Remove(tempPath)
		return "", fmt.Errorf("error reading transcript: %w", err)
	}

	if !foundTargetTurn {
		writer.Flush()
		tempFile.Close()
		os.Remove(tempPath)
		return "", fmt.Errorf("turn %d not found in transcript", turnNumber)
	}

	// Flush and close the temp file
	if err := writer.Flush(); err != nil {
		tempFile.Close()
		os.Remove(tempPath)
		return "", fmt.Errorf("failed to write truncated transcript: %w", err)
	}
	tempFile.Close()

	return tempPath, nil
}

// copyFile copies a file from src to dst
func copyFile(src, dst string) error {
	sourceFile, err := os.Open(src)
	if err != nil {
		return err
	}
	defer sourceFile.Close()

	destFile, err := os.Create(dst)
	if err != nil {
		return err
	}
	defer destFile.Close()

	if _, err := destFile.ReadFrom(sourceFile); err != nil {
		return err
	}

	return destFile.Sync()
}

// RestoreTranscript restores a transcript from its backup
func RestoreTranscript(originalPath, backupPath string) error {
	return copyFile(backupPath, originalPath)
}
