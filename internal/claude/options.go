package claude

// ExecuteOptions contains runtime parameters for Claude execution
type ExecuteOptions struct {
	// Prompt is the initial prompt to send to Claude
	Prompt string
	// Mode is the permission mode for the session
	// Valid options: "acceptEdits", "bypassPermissions", "default", "plan"
	Mode string
	// SystemPrompt is the system prompt to append to the default system prompt
	SystemPrompt string
}

// BuildArgs constructs command line arguments from ExecuteOptions
func (o *ExecuteOptions) BuildArgs(args []string) []string {
	// Add permission mode if specified
	if o.Mode != "" {
		args = append(args, permissionModeFlag, o.Mode)
	}
	// Add system prompt if specified
	if o.SystemPrompt != "" {
		args = append(args, appendSystemPromptFlag, o.SystemPrompt)
	}
	// Add prompt if specified
	if o.Prompt != "" {
		args = append(args, o.Prompt)
	}
	return args
}
