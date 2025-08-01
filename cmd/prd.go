package cmd

import (
	"github.com/spf13/cobra"
)

// prdCmd represents the prd command
var prdCmd = &cobra.Command{
	Use:   "prd",
	Short: "Product Requirements Document management commands",
	Long: `PRD (Product Requirements Document) management commands.
	
This command provides subcommands to help create, manage, and maintain product requirements documents
using Claude AI assistance.`,
}

func init() {
	rootCmd.AddCommand(prdCmd)
}
