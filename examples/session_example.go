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

	// Example 1: ExecuteInteractive - Now automatically tracks session!
	fmt.Println("=== Example 1: Interactive Mode with Automatic Session Tracking ===")
	fmt.Println("The new ExecuteInteractive automatically:")
	fmt.Println("1. Executes your prompt and gets a session ID")
	fmt.Println("2. Shows the initial response")
	fmt.Println("3. Continues in interactive mode with the same session\n")

	err := executor.ExecuteInteractive("Create a simple calculator function in Go")
	if err != nil {
		log.Fatalf("Error in interactive mode: %v", err)
	}

	fmt.Println("\nInteractive session completed.")

	// Example 2: ExecuteAndContinueInteractive - Same as above but returns SessionInfo
	/*
		fmt.Println("\n=== Example 2: Execute and Continue Interactive (with SessionInfo) ===")
		sessionInfo, err := executor.ExecuteAndContinueInteractive("Create a REST API endpoint")
		if err != nil {
			log.Printf("Error: %v", err)
		} else {
			fmt.Printf("\nSession completed. ID: %s, Total cost: $%.6f\n",
				sessionInfo.ID, sessionInfo.CostUSD)
		}
	*/

	// Example 3: Interactive mode with specific session
	/*
		fmt.Println("\n=== Example 3: Interactive Mode with Specific Session ===")
		sessionID := "your-session-id-here"
		err := executor.ExecuteInteractiveWithSession(sessionID)
		if err != nil {
			log.Printf("Error: %v", err)
		}
	*/
}
