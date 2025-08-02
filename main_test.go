package main

import (
	"io"
	"os"
	"testing"

	"github.com/ashigirl96/hail-mary/cmd"
)

// TestMain tests the main function
func TestMain(t *testing.T) {
	// Save original args
	oldArgs := os.Args
	defer func() {
		os.Args = oldArgs
	}()

	// Test with version flag (simple command that exits cleanly)
	t.Run("successful execution", func(t *testing.T) {
		os.Args = []string{"hail-mary", "prd", "--help"}

		// Redirect stdout and stderr to discard output
		oldStdout := os.Stdout
		oldStderr := os.Stderr
		os.Stdout, _ = os.Open(os.DevNull)
		os.Stderr, _ = os.Open(os.DevNull)
		defer func() {
			os.Stdout = oldStdout
			os.Stderr = oldStderr
		}()

		// This should not panic
		main()
	})
}

// TestExecuteFunction tests that cmd.Execute can be called
func TestExecuteFunction(t *testing.T) {
	// Save original args
	oldArgs := os.Args
	defer func() {
		os.Args = oldArgs
	}()

	// Test with help command
	os.Args = []string{"hail-mary", "--help"}

	// Redirect output
	oldStdout := os.Stdout
	oldStderr := os.Stderr
	r, w, _ := os.Pipe()
	os.Stdout = w
	os.Stderr = w
	defer func() {
		w.Close()
		os.Stdout = oldStdout
		os.Stderr = oldStderr
		io.Copy(io.Discard, r)
	}()

	// This should work without error
	cmd.Execute()
}
