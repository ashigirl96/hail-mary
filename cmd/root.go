package cmd

import (
	"fmt"
	"log/slog"
	"os"
	"strings"

	"github.com/spf13/cobra"
)

var (
	// ログレベルフラグ
	logLevel string
	// ロガー
	logger *slog.Logger
)

var rootCmd = &cobra.Command{
	Use:   "hail-mary",
	Short: "A CLI tool with TUI support using Cobra, slog, and Bubbletea",
	Long: `Hail Mary is a modern CLI application that demonstrates:
- Command-line interface with Cobra
- Structured logging with slog
- Terminal UI with Bubbletea for specific subcommands`,
	PersistentPreRun: func(cmd *cobra.Command, args []string) {
		// slogの設定
		setupLogger()
		logger.Info("Starting hail-mary",
			slog.String("command", cmd.Name()),
			slog.Any("args", args),
		)
	},
}

// Execute executes the root command
func Execute() {
	if err := rootCmd.Execute(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

func init() {
	// グローバルフラグの設定
	rootCmd.PersistentFlags().StringVar(&logLevel, "log-level", "info", "Set log level (debug, info, warn, error)")
}

// setupLogger configures slog based on the log level flag
func setupLogger() {
	var level slog.Level
	switch strings.ToLower(logLevel) {
	case "debug":
		level = slog.LevelDebug
	case "info":
		level = slog.LevelInfo
	case "warn":
		level = slog.LevelWarn
	case "error":
		level = slog.LevelError
	default:
		level = slog.LevelInfo
	}

	opts := &slog.HandlerOptions{
		Level: level,
	}

	handler := slog.NewTextHandler(os.Stderr, opts)
	logger = slog.New(handler)
	
	// デフォルトロガーとして設定
	slog.SetDefault(logger)
}

// GetLogger returns the configured logger
func GetLogger() *slog.Logger {
	if logger == nil {
		setupLogger()
	}
	return logger
}