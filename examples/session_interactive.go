//go:build ignore
// +build ignore

package main

import (
	"fmt"
	"log"

	"github.com/ashigirl96/hail-mary/internal/claude"
)

func main() {
	executor := claude.NewExecutor()

	// Step 1: First execute with session tracking to get session ID
	fmt.Println("=== Step 1: Initial execution with session tracking ===")
	fmt.Println("Input: 'Hello'")
	sessionInfo, err := executor.ExecuteWithSessionTracking("Hello")
	if err != nil {
		log.Fatalf("Error executing with session tracking: %v", err)
	}

	fmt.Printf("Session ID: %s\n", sessionInfo.ID)
	fmt.Printf("Cost: $%.6f\n", sessionInfo.CostUSD)
	fmt.Printf("Initial Result:\n%s\n\n", sessionInfo.Result)

	// Step 2: Resume the session in interactive mode
	fmt.Println("=== Step 2: Resuming session in interactive mode ===")
	fmt.Printf("Launching interactive Claude shell for session: %s\n", sessionInfo.ID)
	fmt.Println("You can continue the conversation interactively...")
	fmt.Println("Press Ctrl+C to exit the Claude shell.")

	// This will open an interactive session where you can continue the conversation
	err = executor.ExecuteInteractiveWithSession(sessionInfo.ID)
	if err != nil {
		log.Printf("Error resuming interactive session: %v", err)
	}

	fmt.Println("\nInteractive session completed.")
}
