//go:build ignore
// +build ignore

package main

import (
	"fmt"
	"os"
)

func main() {
	args := os.Args[1:]

	// Check if it's a JSON output request
	hasJSON := false
	hasResume := false
	hasContinue := false
	sessionID := ""

	for i, arg := range args {
		if arg == "--output-format=json" {
			hasJSON = true
		}
		if arg == "--resume" && i+1 < len(args) {
			hasResume = true
			sessionID = args[i+1]
		}
		if arg == "--continue" {
			hasContinue = true
		}
	}

	if hasJSON {
		// Return valid JSON for print mode
		if hasResume {
			fmt.Printf(`{"session_id":"%s","result":"Resumed successfully","cost_usd":0.02,"duration":"2s","turns":2}`, sessionID)
		} else {
			fmt.Printf(`{"session_id":"test-session-123","result":"Test response","cost_usd":0.01,"duration":"1s","turns":1}`)
		}
	} else {
		// Interactive mode - just exit successfully
		if hasContinue {
			fmt.Println("Continuing previous session...")
		} else if hasResume {
			fmt.Printf("Resuming session %s...\n", sessionID)
		} else {
			fmt.Println("Interactive mode started")
		}
	}
}
