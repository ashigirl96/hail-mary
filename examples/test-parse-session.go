package main

import (
	"bufio"
	"fmt"
	"log"
	"os"

	"github.com/ashigirl96/hail-mary/internal/claude/schemas"
)

func main() {
	if len(os.Args) < 2 {
		fmt.Println("Usage: test-parse-session <jsonl-file>")
		os.Exit(1)
	}

	file, err := os.Open(os.Args[1])
	if err != nil {
		log.Fatalf("Failed to open file: %v", err)
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	lineNum := 0
	successCount := 0
	errorCount := 0

	for scanner.Scan() {
		lineNum++
		line := scanner.Text()

		if line == "" {
			continue
		}

		msg, err := schemas.UnmarshalSessionMessage([]byte(line))
		if err != nil {
			fmt.Printf("Line %d: ERROR - %v\n", lineNum, err)
			fmt.Printf("  Content preview: %.100s...\n", line)
			errorCount++
			continue
		}

		successCount++
		uuid := schemas.GetUUID(msg)
		fmt.Printf("Line %d: SUCCESS - Type: %s, UUID: %s\n", lineNum, schemas.GetMessageType(msg), uuid)

		// Show content preview
		content := schemas.ExtractContentText(msg)
		if len(content) > 100 {
			content = content[:100] + "..."
		}
		fmt.Printf("  Content: %s\n", content)

		// Additional info based on type
		switch m := msg.(type) {
		case schemas.UserMessage:
			if sessionID := schemas.GetSessionID(msg); sessionID != "" {
				fmt.Printf("  Session ID: %s\n", sessionID)
			}
			// Check for any special fields
			if m.GitBranch != nil {
				fmt.Printf("  Git Branch: %s\n", *m.GitBranch)
			}
		case schemas.AssistantMessage:
			if timestamp := schemas.GetTimestamp(msg); timestamp != "" {
				fmt.Printf("  Timestamp: %s\n", timestamp)
			}
			// Show model info
			fmt.Printf("  Model: %s\n", m.Message.Model)
			fmt.Printf("  Token Usage: input=%d, output=%d\n",
				m.Message.Usage.InputTokens, m.Message.Usage.OutputTokens)
		case schemas.SystemMessage:
			if m.ToolUseID != nil {
				fmt.Printf("  Tool Use ID: %s\n", *m.ToolUseID)
			}
			if m.Level != nil {
				fmt.Printf("  Level: %s\n", *m.Level)
			}
		}
		fmt.Println()
	}

	if err := scanner.Err(); err != nil {
		log.Fatalf("Error reading file: %v", err)
	}

	fmt.Printf("\nSummary:\n")
	fmt.Printf("Total lines: %d\n", lineNum)
	fmt.Printf("Successfully parsed: %d\n", successCount)
	fmt.Printf("Errors: %d\n", errorCount)
	fmt.Printf("Success rate: %.2f%%\n", float64(successCount)/float64(lineNum)*100)
}
