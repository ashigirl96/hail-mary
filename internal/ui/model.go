package ui

import (
	"fmt"
	"log/slog"

	tea "github.com/charmbracelet/bubbletea"
)

// Model represents the TUI state
type Model struct {
	input     string
	cursor    int
	confirmed bool
	logger    *slog.Logger
}

// NewModel creates a new TUI model
func NewModel(initialText string, logger *slog.Logger) Model {
	return Model{
		input:  initialText,
		cursor: len(initialText),
		logger: logger,
	}
}

// Init initializes the model
func (m Model) Init() tea.Cmd {
	m.logger.Debug("TUI initialized")
	return nil
}

// Update handles incoming events
func (m Model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "esc":
			m.logger.Debug("TUI exit requested")
			return m, tea.Quit

		case "enter":
			m.confirmed = true
			m.logger.Debug("Input confirmed", slog.String("input", m.input))
			return m, tea.Quit

		case "left":
			if m.cursor > 0 {
				m.cursor--
			}

		case "right":
			if m.cursor < len(m.input) {
				m.cursor++
			}

		case "backspace":
			if m.cursor > 0 && len(m.input) > 0 {
				// カーソル位置の前の文字を削除
				m.input = m.input[:m.cursor-1] + m.input[m.cursor:]
				m.cursor--
			}

		case "delete":
			if m.cursor < len(m.input) {
				// カーソル位置の文字を削除
				m.input = m.input[:m.cursor] + m.input[m.cursor+1:]
			}

		case "home":
			m.cursor = 0

		case "end":
			m.cursor = len(m.input)

		default:
			// 通常の文字入力
			if len(msg.String()) == 1 {
				// カーソル位置に文字を挿入
				m.input = m.input[:m.cursor] + msg.String() + m.input[m.cursor:]
				m.cursor++
			}
		}
	}

	return m, nil
}

// View renders the UI
func (m Model) View() string {
	// カーソル位置を表示するために文字列を分割
	var displayText string
	if m.cursor < len(m.input) {
		displayText = m.input[:m.cursor] + "│" + m.input[m.cursor:]
	} else {
		displayText = m.input + "│"
	}

	return fmt.Sprintf(
		"\n🚀 Hail Mary TUI Demo\n\n"+
			"Enter some text:\n"+
			"> %s\n\n"+
			"Commands:\n"+
			"  • Enter: Confirm and exit\n"+
			"  • Esc/Ctrl+C: Cancel and exit\n"+
			"  • ←/→: Move cursor\n"+
			"  • Home/End: Jump to start/end\n"+
			"  • Backspace/Delete: Remove characters\n",
		displayText,
	)
}

// GetInput returns the current input text
func (m Model) GetInput() string {
	return m.input
}

// IsConfirmed returns whether the input was confirmed
func (m Model) IsConfirmed() bool {
	return m.confirmed
}
