package ui

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"

	"github.com/ashigirl96/hail-mary/internal/session"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

// PRDResumeModel represents the TUI state for PRD resume interface
type PRDResumeModel struct {
	features        []string      // List of feature directories
	sessions        []SessionInfo // List of sessions for selected feature
	featureIndex    int           // Current selection in features list
	sessionIndex    int           // Current selection in sessions list
	activePane      int           // 0 = left (features), 1 = right (sessions)
	selectedFeature string        // Selected feature name
	selectedSession string        // Selected session ID
	width           int           // Terminal width
	height          int           // Terminal height
	err             error         // Error state
	loading         bool          // Loading state for sessions
	confirmed       bool          // Whether selection is confirmed
}

// SessionInfo represents a Claude session
type SessionInfo struct {
	ID          string
	StartTime   time.Time
	LastUpdated time.Time
	TurnCount   int
	Summary     string // First user message as summary
}

// NewPRDResumeModel creates a new TUI model for PRD resume
func NewPRDResumeModel(features []string) PRDResumeModel {
	return PRDResumeModel{
		features:     features,
		sessions:     []SessionInfo{},
		featureIndex: 0,
		sessionIndex: 0,
		activePane:   0, // Start with features pane active
	}
}

// Init initializes the model
func (m PRDResumeModel) Init() tea.Cmd {
	// Load sessions for the first feature if available
	if len(m.features) > 0 {
		return m.loadSessionsCmd(m.features[0])
	}
	return nil
}

// Update handles incoming events
func (m PRDResumeModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.width = msg.Width
		m.height = msg.Height
		return m, nil

	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "q", "esc":
			return m, tea.Quit

		case "enter":
			if m.activePane == 1 && m.sessionIndex < len(m.sessions) {
				// Select the current session
				m.selectedFeature = m.features[m.featureIndex]
				m.selectedSession = m.sessions[m.sessionIndex].ID
				m.confirmed = true
				return m, tea.Quit
			}
			return m, nil

		case "h", "left":
			// Move to left pane (features)
			if m.activePane == 1 {
				m.activePane = 0
			}
			return m, nil

		case "l", "right":
			// Move to right pane (sessions)
			if m.activePane == 0 && len(m.sessions) > 0 {
				m.activePane = 1
			}
			return m, nil

		case "j", "down":
			if m.activePane == 0 {
				// Navigate features
				if m.featureIndex < len(m.features)-1 {
					m.featureIndex++
					m.sessionIndex = 0 // Reset session selection
					return m, m.loadSessionsCmd(m.features[m.featureIndex])
				}
			} else if m.activePane == 1 {
				// Navigate sessions
				if m.sessionIndex < len(m.sessions)-1 {
					m.sessionIndex++
				}
			}
			return m, nil

		case "k", "up":
			if m.activePane == 0 {
				// Navigate features
				if m.featureIndex > 0 {
					m.featureIndex--
					m.sessionIndex = 0 // Reset session selection
					return m, m.loadSessionsCmd(m.features[m.featureIndex])
				}
			} else if m.activePane == 1 {
				// Navigate sessions
				if m.sessionIndex > 0 {
					m.sessionIndex--
				}
			}
			return m, nil
		}

	case sessionsLoadedMsg:
		m.sessions = msg.sessions
		m.loading = false
		return m, nil

	case errorMsg:
		m.err = msg.err
		m.loading = false
		return m, nil
	}

	return m, nil
}

// View renders the UI
func (m PRDResumeModel) View() string {
	if m.err != nil {
		return fmt.Sprintf("Error: %v\n\nPress q to quit.", m.err)
	}

	if m.width == 0 || m.height == 0 {
		return "Loading..."
	}

	// Calculate pane dimensions
	paneWidth := (m.width - 3) / 2 // -3 for borders and separator
	paneHeight := m.height - 4     // -4 for header and footer

	// Render left pane (features)
	leftPane := m.renderFeaturesPane(paneWidth, paneHeight)

	// Render right pane (sessions)
	rightPane := m.renderSessionsPane(paneWidth, paneHeight)

	// Join panes horizontally
	content := lipgloss.JoinHorizontal(lipgloss.Top, leftPane, " │ ", rightPane)

	// Add header and footer
	header := lipgloss.NewStyle().
		Bold(true).
		Foreground(lipgloss.Color("205")).
		Render("PRD Resume - Select Feature and Session")

	footer := lipgloss.NewStyle().
		Foreground(lipgloss.Color("241")).
		Render("j/k: navigate • h/l: switch panes • enter: select • q: quit")

	// Combine all parts
	return lipgloss.JoinVertical(lipgloss.Left,
		header,
		"",
		content,
		"",
		footer,
	)
}

// renderFeaturesPane renders the left pane with feature list
func (m PRDResumeModel) renderFeaturesPane(width, height int) string {
	paneStyle := lipgloss.NewStyle().
		Width(width).
		Height(height).
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color(m.getPaneBorderColor(0)))

	title := lipgloss.NewStyle().
		Bold(true).
		Foreground(lipgloss.Color("205")).
		Render("Features")

	var items []string
	items = append(items, title)
	items = append(items, "")

	// Render feature list
	for i, feature := range m.features {
		style := lipgloss.NewStyle()
		prefix := "  "

		if i == m.featureIndex {
			prefix = "> "
			if m.activePane == 0 {
				style = style.Bold(true).Foreground(lipgloss.Color("205"))
			} else {
				style = style.Foreground(lipgloss.Color("250"))
			}
		}

		// Convert kebab-case back to readable format
		displayName := strings.ReplaceAll(feature, "-", " ")
		// Capitalize first letter of each word
		words := strings.Fields(displayName)
		for i, word := range words {
			if len(word) > 0 {
				words[i] = strings.ToUpper(word[:1]) + word[1:]
			}
		}
		displayName = strings.Join(words, " ")

		items = append(items, style.Render(prefix+displayName))
	}

	content := strings.Join(items, "\n")
	return paneStyle.Render(content)
}

// renderSessionsPane renders the right pane with session list
func (m PRDResumeModel) renderSessionsPane(width, height int) string {
	paneStyle := lipgloss.NewStyle().
		Width(width).
		Height(height).
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color(m.getPaneBorderColor(1)))

	title := lipgloss.NewStyle().
		Bold(true).
		Foreground(lipgloss.Color("205")).
		Render("Sessions")

	var items []string
	items = append(items, title)
	items = append(items, "")

	if m.loading {
		items = append(items, "Loading sessions...")
	} else if len(m.sessions) == 0 {
		items = append(items, lipgloss.NewStyle().
			Foreground(lipgloss.Color("241")).
			Render("No sessions found for this feature"))
	} else {
		// Render session list
		for i, session := range m.sessions {
			style := lipgloss.NewStyle()
			prefix := "  "

			if i == m.sessionIndex {
				prefix = "> "
				if m.activePane == 1 {
					style = style.Bold(true).Foreground(lipgloss.Color("205"))
				} else {
					style = style.Foreground(lipgloss.Color("250"))
				}
			}

			// Format session info
			timeStr := session.StartTime.Format("Jan 02 15:04")
			sessionInfo := fmt.Sprintf("%s... (%s, %d turns)",
				session.ID[:8], timeStr, session.TurnCount)

			items = append(items, style.Render(prefix+sessionInfo))

			// Add summary if we're on this item
			if i == m.sessionIndex && session.Summary != "" {
				summaryStyle := lipgloss.NewStyle().
					Foreground(lipgloss.Color("241")).
					PaddingLeft(4)
				summary := truncateString(session.Summary, width-6)
				items = append(items, summaryStyle.Render(summary))
			}
		}
	}

	content := strings.Join(items, "\n")
	return paneStyle.Render(content)
}

// getPaneBorderColor returns the border color for a pane
func (m PRDResumeModel) getPaneBorderColor(pane int) string {
	if m.activePane == pane {
		return "205" // Pink for active
	}
	return "241" // Gray for inactive
}

// loadSessionsCmd returns a command to load sessions for a feature
func (m PRDResumeModel) loadSessionsCmd(feature string) tea.Cmd {
	return func() tea.Msg {
		sessions, err := m.loadSessionsForFeature(feature)
		if err != nil {
			return errorMsg{err: err}
		}
		return sessionsLoadedMsg{sessions: sessions}
	}
}

// loadSessionsForFeature loads Claude sessions related to a feature
func (m PRDResumeModel) loadSessionsForFeature(feature string) ([]SessionInfo, error) {
	// Create feature state manager
	featureDir := filepath.Join(".kiro", "spec", feature)
	featureManager := session.NewFeatureStateManager(featureDir)

	// List all session states in the feature directory
	states, err := featureManager.ListStates()
	if err != nil {
		return nil, fmt.Errorf("failed to list session states: %w", err)
	}

	var sessions []SessionInfo

	for _, state := range states {
		// Parse the transcript file to get session details
		sessionInfo, err := m.parseTranscriptFile(state.TranscriptPath, state)
		if err != nil {
			// Skip files that can't be parsed
			continue
		}

		if sessionInfo != nil {
			sessions = append(sessions, *sessionInfo)
		}
	}

	// Sort sessions by start time (newest first)
	sort.Slice(sessions, func(i, j int) bool {
		return sessions[i].StartTime.After(sessions[j].StartTime)
	})

	return sessions, nil
}

// parseTranscriptFile parses a Claude transcript file using the session state
func (m PRDResumeModel) parseTranscriptFile(path string, state *session.State) (*SessionInfo, error) {
	file, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	var firstUserMessage string
	var turnCount int

	for scanner.Scan() {
		var entry map[string]interface{}
		if err := json.Unmarshal(scanner.Bytes(), &entry); err != nil {
			continue
		}

		// Check for user messages in Claude's JSONL format
		if msgType, ok := entry["type"].(string); ok && msgType == "user" {
			if message, ok := entry["message"].(map[string]interface{}); ok {
				if role, ok := message["role"].(string); ok && role == "user" {
					if content, ok := message["content"].(string); ok {
						// Skip hook messages
						if !strings.Contains(content, "-hook>") && firstUserMessage == "" {
							firstUserMessage = content
						}
						if !strings.Contains(content, "-hook>") {
							turnCount++
						}
					}
				}
			}
		}
	}

	// Since this session is already associated with the feature,
	// we can return it directly
	return &SessionInfo{
		ID:          state.SessionID,
		StartTime:   state.StartedAt,
		LastUpdated: state.LastUpdated,
		TurnCount:   turnCount,
		Summary:     truncateString(firstUserMessage, 100),
	}, nil
}

// truncateString truncates a string to a maximum length
func truncateString(s string, maxLen int) string {
	if len(s) <= maxLen {
		return s
	}
	return s[:maxLen-3] + "..."
}

// Message types for async operations
type sessionsLoadedMsg struct {
	sessions []SessionInfo
}

type errorMsg struct {
	err error
}

// GetSelectedFeature returns the selected feature
func (m PRDResumeModel) GetSelectedFeature() string {
	if m.confirmed {
		return m.selectedFeature
	}
	return ""
}

// GetSelectedSession returns the selected session ID
func (m PRDResumeModel) GetSelectedSession() string {
	if m.confirmed {
		return m.selectedSession
	}
	return ""
}

// RunPRDResume runs the PRD resume UI and returns the selected feature and session
func RunPRDResume(features []string) (string, string, error) {
	model := NewPRDResumeModel(features)
	p := tea.NewProgram(model, tea.WithAltScreen())

	finalModel, err := p.Run()
	if err != nil {
		return "", "", fmt.Errorf("failed to run PRD resume UI: %w", err)
	}

	if m, ok := finalModel.(PRDResumeModel); ok {
		return m.GetSelectedFeature(), m.GetSelectedSession(), nil
	}

	return "", "", fmt.Errorf("unexpected model type")
}
