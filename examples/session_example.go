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

	// Example 3: Programmatic execution (non-interactive)
	/*
		fmt.Println("\n=== Example 3: Programmatic Execution ===")
		sessionInfo, err := executor.ExecuteWithSessionTracking("Explain Go interfaces")
		if err != nil {
			log.Printf("Error: %v", err)
		} else {
			fmt.Printf("Session ID: %s\n", sessionInfo.ID)
			fmt.Printf("Cost: $%.6f\n", sessionInfo.CostUSD)
			fmt.Printf("Result:\n%s\n", sessionInfo.Result)

			// Resume the session programmatically
			resumedInfo, err := executor.ResumeSession(sessionInfo.ID, "Give an example")
			if err != nil {
				log.Printf("Error resuming: %v", err)
			} else {
				fmt.Printf("\nResumed result:\n%s\n", resumedInfo.Result)
			}
		}
	*/
}
