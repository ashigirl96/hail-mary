package prompt

import (
	_ "embed"
	"log/slog"
	"strings"
)

// Embed the PRD system prompt at build time
//
//go:embed prd.md
var prdSystemPrompt string

// ReadPRDSystemPrompt returns the embedded PRD system prompt
func ReadPRDSystemPrompt(logger *slog.Logger) (string, error) {
	// Trim whitespace from the embedded content
	systemPrompt := strings.TrimSpace(prdSystemPrompt)

	logger.Debug("Loaded PRD system prompt from embedded content",
		"length", len(systemPrompt))

	return systemPrompt, nil
}
