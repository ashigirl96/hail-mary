package ui

import (
	"fmt"
	"strings"

	tea "github.com/charmbracelet/bubbletea"
)

// FeatureInputModel represents the TUI state for feature title input
type FeatureInputModel struct {
	input        string
	cursor       int
	confirmed    bool
	featureTitle string
	err          error
}

// NewFeatureInputModel creates a new TUI model for feature input
func NewFeatureInputModel() FeatureInputModel {
	return FeatureInputModel{
		input:  "",
		cursor: 0,
	}
}

// Init initializes the model
func (m FeatureInputModel) Init() tea.Cmd {
	return nil
}

// Update handles incoming events
func (m FeatureInputModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "esc":
			m.err = fmt.Errorf("cancelled by user")
			return m, tea.Quit

		case "enter":
			trimmed := strings.TrimSpace(m.input)
			if trimmed == "" {
				// Don't accept empty input
				return m, nil
			}
			m.featureTitle = trimmed
			m.confirmed = true
			return m, tea.Quit

		case "left", "ctrl+a":
			if m.cursor > 0 {
				m.cursor--
			}

		case "right", "ctrl+e":
			if m.cursor < len(m.input) {
				m.cursor++
			}

		case "backspace", "ctrl+h":
			if m.cursor > 0 && len(m.input) > 0 {
				// Remove character before cursor
				m.input = m.input[:m.cursor-1] + m.input[m.cursor:]
				m.cursor--
			}

		case "delete", "ctrl+d":
			if m.cursor < len(m.input) {
				// Remove character at cursor
				m.input = m.input[:m.cursor] + m.input[m.cursor+1:]
			}

		case "home":
			m.cursor = 0

		case "end":
			m.cursor = len(m.input)

		case "ctrl+u":
			// Clear line
			m.input = ""
			m.cursor = 0

		case "ctrl+k":
			// Kill to end of line
			m.input = m.input[:m.cursor]

		default:
			// Regular character input
			if len(msg.String()) == 1 && !strings.Contains(msg.String(), "/") {
				// Insert character at cursor position (exclude / for directory safety)
				m.input = m.input[:m.cursor] + msg.String() + m.input[m.cursor:]
				m.cursor++
			}
		}
	}

	return m, nil
}

// View renders the UI
func (m FeatureInputModel) View() string {
	// Show cursor position
	var displayText string
	if m.cursor < len(m.input) {
		displayText = m.input[:m.cursor] + "│" + m.input[m.cursor:]
	} else {
		displayText = m.input + "│"
	}

	return fmt.Sprintf(
		"\n📋 PRD Feature Title\n\n"+
			"Enter a title for your feature (this will be used as the directory name):\n\n"+
			"> %s\n\n"+
			"Press Enter to confirm, or Esc/Ctrl+C to cancel.\n",
		displayText,
	)
}

// GetFeatureTitle returns the entered feature title
func (m FeatureInputModel) GetFeatureTitle() (string, error) {
	if m.err != nil {
		return "", m.err
	}
	if !m.confirmed {
		return "", fmt.Errorf("input not confirmed")
	}
	return m.featureTitle, nil
}

// RunFeatureInput runs the feature input UI and returns the entered title
func RunFeatureInput() (string, error) {
	model := NewFeatureInputModel()
	p := tea.NewProgram(model)

	finalModel, err := p.Run()
	if err != nil {
		return "", fmt.Errorf("failed to run feature input UI: %w", err)
	}

	if m, ok := finalModel.(FeatureInputModel); ok {
		return m.GetFeatureTitle()
	}

	return "", fmt.Errorf("unexpected model type")
}
