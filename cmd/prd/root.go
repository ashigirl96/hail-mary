package prd

import (
	"github.com/spf13/cobra"
)

// PrdCmd represents the prd command
var PrdCmd = &cobra.Command{
	Use:   "prd",
	Short: "Product Requirements Document management commands",
	Long: `PRD (Product Requirements Document) management commands.
	
This command provides subcommands to help create, manage, and maintain product requirements documents
using Claude AI assistance.`,
}

func Init(rootCmd *cobra.Command) {
	rootCmd.AddCommand(PrdCmd)
}