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

	"github.com/ashigirl96/hail-mary/internal/claude"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

// PRDResumeModel represents the TUI state for PRD resume interface
type PRDResumeModel struct {
	features        []string      // List of feature directories
	sessions        []SessionInfo // List of sessions for selected feature
	featureIndex    int           // Current selection in features list
	sessionIndex    int           // Current selection in sessions list
	activePane      int           // 0 = left (markdown), 1 = right (sessions)
	selectedFeature string        // Selected feature name
	selectedSession string        // Selected session ID
	width           int           // Terminal width
	height          int           // Terminal height
	err             error         // Error state
	loading         bool          // Loading state for sessions
	confirmed       bool          // Whether selection is confirmed
	// Markdown preview fields
	markdownContent      string   // Content of requirement.md
	markdownLines        []string // Rendered markdown lines
	markdownScrollOffset int      // Scroll position in markdown preview
	markdownLoading      bool     // Loading state for markdown
}

// UserInput represents a user input in a Claude session
type UserInput struct {
	Content    string    // The user input content
	Timestamp  time.Time // When the input was made
	TurnNumber int       // Turn number in the session
}

// SessionInfo represents a Claude session
type SessionInfo struct {
	ID          string      // Session ID
	StartTime   time.Time   // When the session started
	LastUpdated time.Time   // Last update time
	TurnCount   int         // Number of turns
	Summary     string      // First user message as summary
	UserInputs  []UserInput // All user inputs in the session
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
	// Load sessions and markdown for the first feature if available
	if len(m.features) > 0 {
		return tea.Batch(
			m.loadSessionsCmd(m.features[0]),
			m.loadMarkdownCmd(m.features[0]),
		)
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

		case "ctrl+u":
			// Scroll markdown preview up
			if m.activePane == 0 && len(m.markdownLines) > 0 {
				if m.markdownScrollOffset > 0 {
					m.markdownScrollOffset -= 5 // Scroll up by 5 lines
					if m.markdownScrollOffset < 0 {
						m.markdownScrollOffset = 0
					}
				}
			}
			return m, nil

		case "ctrl+d":
			// Scroll markdown preview down
			if m.activePane == 0 && len(m.markdownLines) > 0 {
				paneHeight := m.height - 6
				maxScroll := len(m.markdownLines) - paneHeight
				if maxScroll > 0 && m.markdownScrollOffset < maxScroll {
					m.markdownScrollOffset += 5 // Scroll down by 5 lines
					if m.markdownScrollOffset > maxScroll {
						m.markdownScrollOffset = maxScroll
					}
				}
			}
			return m, nil

		case "j", "down":
			if m.activePane == 0 {
				// Navigate features
				if m.featureIndex < len(m.features)-1 {
					m.featureIndex++
					m.sessionIndex = 0         // Reset session selection
					m.markdownScrollOffset = 0 // Reset scroll
					return m, tea.Batch(
						m.loadSessionsCmd(m.features[m.featureIndex]),
						m.loadMarkdownCmd(m.features[m.featureIndex]),
					)
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
					m.sessionIndex = 0         // Reset session selection
					m.markdownScrollOffset = 0 // Reset scroll
					return m, tea.Batch(
						m.loadSessionsCmd(m.features[m.featureIndex]),
						m.loadMarkdownCmd(m.features[m.featureIndex]),
					)
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

	case markdownLoadedMsg:
		m.markdownContent = msg.content
		m.markdownLines = msg.lines
		m.markdownLoading = false
		return m, nil

	case errorMsg:
		m.err = msg.err
		m.loading = false
		m.markdownLoading = false
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

	// Render left pane (markdown preview)
	leftPane := m.renderMarkdownPane(paneWidth, paneHeight)

	// Render right pane (sessions with user inputs)
	rightPane := m.renderSessionsPane(paneWidth, paneHeight)

	// Join panes horizontally
	content := lipgloss.JoinHorizontal(lipgloss.Top, leftPane, " │ ", rightPane)

	// Add header and footer with current feature info
	currentFeature := ""
	if len(m.features) > 0 && m.featureIndex < len(m.features) {
		currentFeature = strings.ReplaceAll(m.features[m.featureIndex], "-", " ")
		words := strings.Fields(currentFeature)
		for i, word := range words {
			if len(word) > 0 {
				words[i] = strings.ToUpper(word[:1]) + word[1:]
			}
		}
		currentFeature = strings.Join(words, " ")
	}

	header := lipgloss.NewStyle().
		Bold(true).
		Foreground(lipgloss.Color("205")).
		Render(fmt.Sprintf("PRD Resume - %s", currentFeature))

	footer := lipgloss.NewStyle().
		Foreground(lipgloss.Color("241")).
		Render("j/k: navigate features • h/l: switch panes • ctrl+u/d: scroll markdown • enter: select • q: quit")

	// Combine all parts
	return lipgloss.JoinVertical(lipgloss.Left,
		header,
		"",
		content,
		"",
		footer,
	)
}

// renderMarkdownPane renders the left pane with markdown preview
func (m PRDResumeModel) renderMarkdownPane(width, height int) string {
	paneStyle := lipgloss.NewStyle().
		Width(width).
		Height(height).
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color(m.getPaneBorderColor(0)))

	title := lipgloss.NewStyle().
		Bold(true).
		Foreground(lipgloss.Color("205")).
		Render("Requirements Preview")

	var items []string
	items = append(items, title)
	items = append(items, "")

	if m.markdownLoading {
		items = append(items, "Loading markdown...")
	} else if len(m.markdownLines) == 0 {
		items = append(items, lipgloss.NewStyle().
			Foreground(lipgloss.Color("241")).
			Render("No requirements.md found for this feature"))
		items = append(items, "")
		items = append(items, lipgloss.NewStyle().
			Foreground(lipgloss.Color("241")).
			Render("Create one to see a preview here."))
	} else {
		// Render markdown with scrolling
		contentHeight := height - 4 // Account for title, border, and padding

		startLine := m.markdownScrollOffset
		endLine := startLine + contentHeight

		if endLine > len(m.markdownLines) {
			endLine = len(m.markdownLines)
		}

		if startLine < len(m.markdownLines) {
			displayLines := m.markdownLines[startLine:endLine]
			items = append(items, displayLines...)
		}

		// Add scroll indicator if content is scrollable
		if len(m.markdownLines) > contentHeight {
			scrollInfo := fmt.Sprintf("[%d-%d/%d]",
				startLine+1,
				endLine,
				len(m.markdownLines))
			indicator := lipgloss.NewStyle().
				Foreground(lipgloss.Color("241")).
				Render(scrollInfo)

			// Replace the title line to include scroll indicator
			items[0] = lipgloss.JoinHorizontal(lipgloss.Left,
				title,
				" ",
				indicator)
		}
	}

	content := strings.Join(items, "\n")
	return paneStyle.Render(content)
}

// renderSessionsPane renders the right pane with session list and user inputs
func (m PRDResumeModel) renderSessionsPane(width, height int) string {
	paneStyle := lipgloss.NewStyle().
		Width(width).
		Height(height).
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color(m.getPaneBorderColor(1)))

	title := lipgloss.NewStyle().
		Bold(true).
		Foreground(lipgloss.Color("205")).
		Render("Session History")

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
		// Render session list with user inputs
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

			// Add user inputs if we're on this item
			if i == m.sessionIndex && len(session.UserInputs) > 0 {
				items = append(items, "")

				inputStyle := lipgloss.NewStyle().
					Foreground(lipgloss.Color("250")).
					PaddingLeft(4)

				inputHeaderStyle := lipgloss.NewStyle().
					Foreground(lipgloss.Color("33")).
					PaddingLeft(4).
					Bold(true)

				items = append(items, inputHeaderStyle.Render("User Inputs:"))

				// Show up to 5 most recent inputs
				maxInputs := 5
				startIdx := 0
				if len(session.UserInputs) > maxInputs {
					startIdx = len(session.UserInputs) - maxInputs
				}

				for j := startIdx; j < len(session.UserInputs); j++ {
					input := session.UserInputs[j]

					// Truncate and format the input
					maxWidth := width - 8 // Account for padding and borders
					truncated := truncateString(input.Content, maxWidth)

					inputLine := fmt.Sprintf("• %s", truncated)
					items = append(items, inputStyle.Render(inputLine))
				}

				// Show count if there are more inputs
				if len(session.UserInputs) > maxInputs {
					moreStyle := lipgloss.NewStyle().
						Foreground(lipgloss.Color("241")).
						PaddingLeft(4).
						Italic(true)
					items = append(items, moreStyle.Render(fmt.Sprintf("... and %d more inputs", len(session.UserInputs)-maxInputs)))
				}
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

// loadMarkdownCmd returns a command to load markdown content for a feature
func (m PRDResumeModel) loadMarkdownCmd(feature string) tea.Cmd {
	return func() tea.Msg {
		content, lines, err := m.loadMarkdownForFeature(feature, m.width/2-6)
		if err != nil {
			// Return empty content if file doesn't exist or can't be read
			return markdownLoadedMsg{
				content: "No requirements.md found for this feature.\n\nCreate one to see a preview here.",
				lines:   []string{"No requirements.md found for this feature.", "", "Create one to see a preview here."},
			}
		}
		return markdownLoadedMsg{content: content, lines: lines}
	}
}

// loadSessionsForFeature loads Claude sessions related to a feature
func (m PRDResumeModel) loadSessionsForFeature(feature string) ([]SessionInfo, error) {
	// Create feature session state manager
	featureDir := filepath.Join(".kiro", "spec", feature)
	sessionManager := claude.NewFeatureSessionStateManager(featureDir)

	// Load all sessions from sessions.json
	sessionsState, err := sessionManager.LoadSessions()
	if err != nil {
		return nil, fmt.Errorf("failed to load sessions: %w", err)
	}

	var sessions []SessionInfo

	for _, state := range sessionsState {
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

// loadMarkdownForFeature loads and renders markdown content for a feature
func (m PRDResumeModel) loadMarkdownForFeature(feature string, width int) (string, []string, error) {
	// Build path to requirements.md
	requirementsPath := filepath.Join(".kiro", "spec", feature, "requirements.md")

	// Read file content
	content, err := os.ReadFile(requirementsPath)
	if err != nil {
		return "", nil, err
	}

	contentStr := string(content)
	lines := renderMarkdown(contentStr, width)

	return contentStr, lines, nil
}

// parseTranscriptFile parses a Claude transcript file using the session state
func (m PRDResumeModel) parseTranscriptFile(path string, state *claude.SessionState) (*SessionInfo, error) {
	file, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	var firstUserMessage string
	var userInputs []UserInput
	var turnCount int
	currentTurn := 0

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
						if !strings.Contains(content, "-hook>") {
							if firstUserMessage == "" {
								firstUserMessage = content
							}

							// Add to user inputs list
							userInputs = append(userInputs, UserInput{
								Content:    content,
								Timestamp:  state.StartedAt, // Use session start time as approximation
								TurnNumber: currentTurn,
							})

							turnCount++
							currentTurn++
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
		UserInputs:  userInputs,
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

type markdownLoadedMsg struct {
	content string
	lines   []string
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

// Markdown rendering functions

// renderMarkdown renders basic markdown to styled strings
func renderMarkdown(content string, width int) []string {
	lines := strings.Split(content, "\n")
	var rendered []string

	for _, line := range lines {
		rendered = append(rendered, renderMarkdownLine(line, width)...)
	}

	return rendered
}

// renderMarkdownLine renders a single markdown line with basic formatting
func renderMarkdownLine(line string, width int) []string {
	line = strings.TrimRight(line, " \t")

	// Handle different markdown elements
	if strings.HasPrefix(line, "# ") {
		// H1 header
		text := strings.TrimPrefix(line, "# ")
		style := lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("205"))
		return wrapText(style.Render("# "+text), width)
	} else if strings.HasPrefix(line, "## ") {
		// H2 header
		text := strings.TrimPrefix(line, "## ")
		style := lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("33"))
		return wrapText(style.Render("## "+text), width)
	} else if strings.HasPrefix(line, "### ") {
		// H3 header
		text := strings.TrimPrefix(line, "### ")
		style := lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("39"))
		return wrapText(style.Render("### "+text), width)
	} else if strings.HasPrefix(line, "- ") || strings.HasPrefix(line, "* ") {
		// Bullet list
		text := line[2:]
		style := lipgloss.NewStyle().Foreground(lipgloss.Color("250"))
		bullet := lipgloss.NewStyle().Foreground(lipgloss.Color("33")).Render("•")
		return wrapText(bullet+" "+style.Render(text), width)
	} else if strings.HasPrefix(line, "```") {
		// Code block
		style := lipgloss.NewStyle().Foreground(lipgloss.Color("241")).Background(lipgloss.Color("234"))
		return []string{style.Render(line)}
	} else if strings.HasPrefix(line, "> ") {
		// Blockquote
		text := strings.TrimPrefix(line, "> ")
		style := lipgloss.NewStyle().Foreground(lipgloss.Color("241")).Italic(true)
		prefix := lipgloss.NewStyle().Foreground(lipgloss.Color("241")).Render("│ ")
		return wrapText(prefix+style.Render(text), width)
	} else if line == "" {
		// Empty line
		return []string{""}
	} else {
		// Regular text with inline formatting
		rendered := renderInlineMarkdown(line)
		return wrapText(rendered, width)
	}
}

// renderInlineMarkdown handles inline markdown formatting
func renderInlineMarkdown(text string) string {
	// Handle bold text **text**
	for {
		if !strings.Contains(text, "**") {
			break
		}
		start := strings.Index(text, "**")
		if start == -1 {
			break
		}
		end := strings.Index(text[start+2:], "**")
		if end == -1 {
			break
		}
		end += start + 2

		before := text[:start]
		content := text[start+2 : end]
		after := text[end+2:]

		styled := lipgloss.NewStyle().Bold(true).Render(content)
		text = before + styled + after
	}

	// Handle italic text *text*
	for {
		if !strings.Contains(text, "*") {
			break
		}
		start := strings.Index(text, "*")
		if start == -1 {
			break
		}
		end := strings.Index(text[start+1:], "*")
		if end == -1 {
			break
		}
		end += start + 1

		before := text[:start]
		content := text[start+1 : end]
		after := text[end+1:]

		styled := lipgloss.NewStyle().Italic(true).Render(content)
		text = before + styled + after
	}

	// Handle code `text`
	for {
		if !strings.Contains(text, "`") {
			break
		}
		start := strings.Index(text, "`")
		if start == -1 {
			break
		}
		end := strings.Index(text[start+1:], "`")
		if end == -1 {
			break
		}
		end += start + 1

		before := text[:start]
		content := text[start+1 : end]
		after := text[end+1:]

		styled := lipgloss.NewStyle().Foreground(lipgloss.Color("203")).Background(lipgloss.Color("234")).Render(content)
		text = before + styled + after
	}

	return text
}

// wrapText wraps text to fit within the specified width
func wrapText(text string, width int) []string {
	if width <= 0 {
		return []string{text}
	}

	words := strings.Fields(text)
	if len(words) == 0 {
		return []string{text}
	}

	var lines []string
	var currentLine []string
	currentLength := 0

	for _, word := range words {
		wordLen := len(word)
		if currentLength+wordLen+len(currentLine) > width && len(currentLine) > 0 {
			lines = append(lines, strings.Join(currentLine, " "))
			currentLine = []string{word}
			currentLength = wordLen
		} else {
			currentLine = append(currentLine, word)
			currentLength += wordLen
		}
	}

	if len(currentLine) > 0 {
		lines = append(lines, strings.Join(currentLine, " "))
	}

	return lines
}
