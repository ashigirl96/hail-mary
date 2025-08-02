package cmd

import (
	"log/slog"
	"os"
	"strings"
	"testing"

	"github.com/spf13/cobra"
)

// TestExecute tests the Execute function
func TestExecute(t *testing.T) {
	// Save original values
	oldArgs := os.Args
	oldStderr := os.Stderr
	oldExitFunc := osExit

	// Track if exit was called
	exitCalled := false
	osExit = func(code int) {
		exitCalled = true
	}

	defer func() {
		// Restore original values
		os.Args = oldArgs
		os.Stderr = oldStderr
		osExit = oldExitFunc
	}()

	// Test successful execution
	os.Args = []string{"hail-mary", "--help"}

	Execute()

	if exitCalled {
		t.Error("Execute() called os.Exit on successful execution")
	}

	// Test failed execution with invalid flag
	exitCalled = false
	os.Args = []string{"hail-mary", "--invalid-flag"}

	Execute()

	if !exitCalled {
		t.Error("Execute() did not call os.Exit on error")
	}
}

// TestSetupLogger tests the logger setup with different log levels
func TestSetupLogger(t *testing.T) {
	tests := []struct {
		name     string
		logLevel string
		expected slog.Level
	}{
		{"debug level", "debug", slog.LevelDebug},
		{"info level", "info", slog.LevelInfo},
		{"warn level", "warn", slog.LevelWarn},
		{"error level", "error", slog.LevelError},
		{"default level", "invalid", slog.LevelInfo},
		{"uppercase level", "DEBUG", slog.LevelDebug},
		{"mixed case level", "WaRn", slog.LevelWarn},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Set log level
			logLevel = tt.logLevel

			// Setup logger
			setupLogger()

			// Verify logger is not nil
			if logger == nil {
				t.Fatal("setupLogger() did not create logger")
			}

			// Test that the logger works
			// This is a smoke test to ensure no panic
			logger.Info("test message")
			logger.Debug("debug message")
			logger.Warn("warn message")
			logger.Error("error message")

			// Verify slog default is set
			if slog.Default() != logger {
				t.Error("setupLogger() did not set default logger")
			}
		})
	}
}

// TestGetLogger tests the GetLogger function
func TestGetLogger(t *testing.T) {
	// Test when logger is nil
	logger = nil
	got := GetLogger()

	if got == nil {
		t.Fatal("GetLogger() returned nil")
	}

	// Verify it's the same instance
	got2 := GetLogger()
	if got != got2 {
		t.Error("GetLogger() returned different instances")
	}

	// Test when logger already exists
	existingLogger := slog.New(slog.NewTextHandler(os.Stderr, nil))
	logger = existingLogger

	got3 := GetLogger()
	if got3 != existingLogger {
		t.Error("GetLogger() did not return existing logger")
	}
}

// TestRootCommand tests the root command setup
func TestRootCommand(t *testing.T) {
	// Test command properties
	if rootCmd.Use != "hail-mary" {
		t.Errorf("rootCmd.Use = %q, want %q", rootCmd.Use, "hail-mary")
	}

	if rootCmd.Short == "" {
		t.Error("rootCmd.Short is empty")
	}

	if rootCmd.Long == "" {
		t.Error("rootCmd.Long is empty")
	}

	// Test that PersistentPreRun is set
	if rootCmd.PersistentPreRun == nil {
		t.Error("rootCmd.PersistentPreRun is nil")
	}

	// Test PersistentPreRun execution
	logger = nil // Reset logger

	rootCmd.PersistentPreRun(rootCmd, []string{"arg1", "arg2"})

	// Verify logger was set up
	if logger == nil {
		t.Error("PersistentPreRun did not set up logger")
	}
}

// TestPersistentFlags tests that persistent flags are set correctly
func TestPersistentFlags(t *testing.T) {
	// Test log-level flag
	flag := rootCmd.PersistentFlags().Lookup("log-level")
	if flag == nil {
		t.Fatal("log-level flag not found")
	}

	if flag.DefValue != "info" {
		t.Errorf("log-level default = %q, want %q", flag.DefValue, "info")
	}

	if flag.Usage != "Set log level (debug, info, warn, error)" {
		t.Errorf("log-level usage = %q", flag.Usage)
	}
}

// TestCompletionOptions tests that completion options are set correctly
func TestCompletionOptions(t *testing.T) {
	if !rootCmd.CompletionOptions.HiddenDefaultCmd {
		t.Error("CompletionOptions.HiddenDefaultCmd = false, want true")
	}

	if rootCmd.CompletionOptions.DisableDefaultCmd {
		t.Error("CompletionOptions.DisableDefaultCmd = true, want false")
	}

	if rootCmd.CompletionOptions.DisableNoDescFlag {
		t.Error("CompletionOptions.DisableNoDescFlag = true, want false")
	}

	if rootCmd.CompletionOptions.DisableDescriptions {
		t.Error("CompletionOptions.DisableDescriptions = true, want false")
	}
}

// TestGetFlagCompletions tests the GetFlagCompletions function
func TestGetFlagCompletions(t *testing.T) {
	// Create a test command with various flags
	testCmd := &cobra.Command{
		Use: "test",
	}

	// Add local flags
	testCmd.Flags().String("local-flag", "", "A local flag")
	testCmd.Flags().StringP("short-flag", "s", "", "A flag with shorthand")
	testCmd.Flags().Bool("bool-flag", false, "A boolean flag")

	// Add inherited flags
	testCmd.InheritedFlags().String("inherited-flag", "", "An inherited flag")

	// Get completions
	completions := GetFlagCompletions(testCmd)

	// Check that all flags are included
	expectedFlags := []string{
		"--local-flag",
		"--short-flag",
		"-s",
		"--bool-flag",
		"--inherited-flag",
	}

	for _, expected := range expectedFlags {
		found := false
		for _, completion := range completions {
			if strings.HasPrefix(completion, expected) {
				found = true
				break
			}
		}
		if !found {
			t.Errorf("Expected flag %q not found in completions", expected)
		}
	}

	// Test with descriptions
	for _, completion := range completions {
		if strings.HasPrefix(completion, "--local-flag") {
			if !strings.Contains(completion, "A local flag") {
				t.Error("local-flag completion missing description")
			}
		}
		if strings.HasPrefix(completion, "--short-flag") {
			if !strings.Contains(completion, "A flag with shorthand") {
				t.Error("short-flag completion missing description")
			}
		}
	}

	// Test with command that has overlapping local and inherited flags
	overlapCmd := &cobra.Command{
		Use: "overlap",
	}
	overlapCmd.Flags().String("same-flag", "", "Local version")
	overlapCmd.InheritedFlags().String("same-flag", "", "Inherited version")
	overlapCmd.InheritedFlags().String("unique-inherited", "", "Unique inherited")

	overlapCompletions := GetFlagCompletions(overlapCmd)

	// Count occurrences of same-flag
	sameFlagCount := 0
	for _, completion := range overlapCompletions {
		if strings.HasPrefix(completion, "--same-flag") {
			sameFlagCount++
		}
	}

	if sameFlagCount != 1 {
		t.Errorf("same-flag appeared %d times, want 1", sameFlagCount)
	}

	// Ensure unique-inherited is included
	hasUniqueInherited := false
	for _, completion := range overlapCompletions {
		if strings.HasPrefix(completion, "--unique-inherited") {
			hasUniqueInherited = true
			break
		}
	}

	if !hasUniqueInherited {
		t.Error("unique-inherited flag not found in completions")
	}
}

// TestRootInit tests the init function
func TestRootInit(t *testing.T) {
	// Test that command sorting is disabled
	if cobra.EnableCommandSorting {
		t.Error("cobra.EnableCommandSorting = true, want false")
	}
}

// Mock os.Exit for testing
var osExit = os.Exit

// TestFlagVisitAll tests edge cases in GetFlagCompletions
func TestFlagVisitAll(t *testing.T) {
	// Test with no flags
	emptyCmd := &cobra.Command{Use: "empty"}
	completions := GetFlagCompletions(emptyCmd)

	if len(completions) != 0 {
		t.Errorf("Expected 0 completions for command with no flags, got %d", len(completions))
	}

	// Test with flag without description
	noDescCmd := &cobra.Command{Use: "nodesc"}
	noDescCmd.Flags().String("no-desc-flag", "", "")

	completions = GetFlagCompletions(noDescCmd)

	found := false
	for _, completion := range completions {
		if strings.HasPrefix(completion, "--no-desc-flag") {
			found = true
			// Should not have tab character if no description
			if strings.Contains(completion, "\t\t") {
				t.Error("Flag without description should not have double tabs")
			}
		}
	}

	if !found {
		t.Error("Flag without description not found in completions")
	}
}

// TestRootCmdExecuteWithArgs tests root command execution with various arguments
func TestRootCmdExecuteWithArgs(t *testing.T) {
	// Save original stderr
	oldStderr := os.Stderr
	defer func() {
		os.Stderr = oldStderr
	}()

	tests := []struct {
		name    string
		args    []string
		wantErr bool
	}{
		{
			name:    "no args",
			args:    []string{},
			wantErr: false,
		},
		{
			name:    "help flag",
			args:    []string{"--help"},
			wantErr: false,
		},
		{
			name:    "valid log level",
			args:    []string{"--log-level", "debug"},
			wantErr: false,
		},
		{
			name:    "invalid flag",
			args:    []string{"--unknown-flag"},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Reset command for each test
			rootCmd.SetArgs(tt.args)

			// Execute command
			err := rootCmd.Execute()

			if (err != nil) != tt.wantErr {
				t.Errorf("rootCmd.Execute() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

// TestLoggerConcurrency tests that logger setup is safe for concurrent access
func TestLoggerConcurrency(t *testing.T) {
	// Reset logger
	logger = nil

	// Run GetLogger concurrently
	done := make(chan bool, 10)
	for i := 0; i < 10; i++ {
		go func() {
			l := GetLogger()
			if l == nil {
				t.Error("GetLogger() returned nil in concurrent access")
			}
			done <- true
		}()
	}

	// Wait for all goroutines
	for i := 0; i < 10; i++ {
		<-done
	}

	// Verify logger is set
	if logger == nil {
		t.Error("logger is nil after concurrent access")
	}
}

// Benchmark tests
func BenchmarkGetFlagCompletions(b *testing.B) {
	// Create a command with many flags
	cmd := &cobra.Command{Use: "bench"}
	for i := 0; i < 20; i++ {
		name := "flag-" + string(rune('a'+i))
		cmd.Flags().String(name, "", "Description for "+name)
		if i%2 == 0 {
			cmd.Flags().StringP(name+"-short", string(rune('a'+i)), "", "Short flag")
		}
	}

	for i := 0; i < 10; i++ {
		name := "inherited-" + string(rune('a'+i))
		cmd.InheritedFlags().String(name, "", "Inherited "+name)
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = GetFlagCompletions(cmd)
	}
}

func BenchmarkSetupLogger(b *testing.B) {
	for i := 0; i < b.N; i++ {
		logLevel = "info"
		setupLogger()
	}
}
