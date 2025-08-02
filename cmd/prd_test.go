package cmd

import (
	"testing"
)

// TestPrdCommand tests the prd command setup
func TestPrdCommand(t *testing.T) {
	// Test command properties
	if prdCmd.Use != "prd" {
		t.Errorf("prdCmd.Use = %q, want %q", prdCmd.Use, "prd")
	}

	expectedShort := "Product Requirements Document management commands"
	if prdCmd.Short != expectedShort {
		t.Errorf("prdCmd.Short = %q, want %q", prdCmd.Short, expectedShort)
	}

	if prdCmd.Long == "" {
		t.Error("prdCmd.Long is empty")
	}

	// Test that prd command is registered with root
	found := false
	for _, cmd := range rootCmd.Commands() {
		if cmd.Use == "prd" {
			found = true
			break
		}
	}
	if !found {
		t.Error("prd command not found in root commands")
	}
}

// TestPrdSubcommands tests that prd has expected subcommands
func TestPrdSubcommands(t *testing.T) {
	expectedSubcommands := []string{"init", "continue"}

	subcommandMap := make(map[string]bool)
	for _, cmd := range prdCmd.Commands() {
		subcommandMap[cmd.Use] = true
	}

	for _, expected := range expectedSubcommands {
		if !subcommandMap[expected] {
			t.Errorf("Expected subcommand %q not found in prd commands", expected)
		}
	}
}

// TestPrdInit tests that the prd command is properly initialized
func TestPrdInit(t *testing.T) {
	// Since init() is called automatically when the package loads,
	// we just verify that the prd command is registered with root

	// Verify prd command was added
	found := false
	for _, cmd := range rootCmd.Commands() {
		if cmd.Use == "prd" {
			found = true
			break
		}
	}

	if !found {
		t.Error("prd command not registered with root")
	}
}
