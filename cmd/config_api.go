package cmd

import (
	"fmt"
	"log/slog"

	"github.com/spf13/cobra"
)

// configApiCmd represents the config api command
var configApiCmd = &cobra.Command{
	Use:   "api",
	Short: "Manage API configuration",
	Long:  `Manage API-related configuration settings such as endpoints, timeouts, and authentication.`,
	Run: func(cmd *cobra.Command, args []string) {
		// If no subcommand is provided, show help
		cmd.Help()
	},
}

// configApiEndpointCmd represents the config api endpoint command
var configApiEndpointCmd = &cobra.Command{
	Use:   "endpoint [command]",
	Short: "Manage API endpoint configuration",
	Long:  `Get or set API endpoint configuration.`,
	Run: func(cmd *cobra.Command, args []string) {
		cmd.Help()
	},
}

// configApiEndpointGetCmd represents the config api endpoint get command
var configApiEndpointGetCmd = &cobra.Command{
	Use:   "get",
	Short: "Get current API endpoint",
	Run: func(cmd *cobra.Command, args []string) {
		logger := GetLogger()
		logger.Debug("Getting API endpoint")
		fmt.Println("Current API endpoint: https://api.example.com")
	},
}

// configApiEndpointSetCmd represents the config api endpoint set command
var configApiEndpointSetCmd = &cobra.Command{
	Use:   "set [url]",
	Short: "Set API endpoint",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		logger := GetLogger()
		endpoint := args[0]
		logger.Info("Setting API endpoint", slog.String("endpoint", endpoint))
		fmt.Printf("API endpoint set to: %s\n", endpoint)
	},
}

// configApiTimeoutCmd represents the config api timeout command
var configApiTimeoutCmd = &cobra.Command{
	Use:   "timeout [seconds]",
	Short: "Get or set API timeout",
	Long:  `Get the current API timeout or set a new timeout value in seconds.`,
	Args:  cobra.MaximumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		logger := GetLogger()
		
		if len(args) == 0 {
			// Get current timeout
			logger.Debug("Getting API timeout")
			fmt.Println("Current API timeout: 30s")
		} else {
			// Set new timeout
			timeout := args[0]
			logger.Info("Setting API timeout", slog.String("timeout", timeout))
			fmt.Printf("API timeout set to: %s seconds\n", timeout)
		}
	},
}

func init() {
	// Add api subcommand to config
	configCmd.AddCommand(configApiCmd)
	
	// Add subcommands to config api
	configApiCmd.AddCommand(configApiEndpointCmd)
	configApiCmd.AddCommand(configApiTimeoutCmd)
	
	// Add subcommands to config api endpoint
	configApiEndpointCmd.AddCommand(configApiEndpointGetCmd)
	configApiEndpointCmd.AddCommand(configApiEndpointSetCmd)
	
}