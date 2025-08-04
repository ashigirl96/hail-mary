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
	features        []string        // List of feature directories
	sessions        []SessionInfo   // List of sessions for selected feature (kept for compatibility)
	flatInputs      []FlatUserInput // Flattened list of all user inputs from all sessions
	featureIndex    int             // Current selection in features list
	inputIndex      int             // Current selection in flat inputs list (0 = continue, 1+ = user inputs)
	activePane      int             // 0 = left (markdown), 1 = right (inputs)
	selectedFeature string          // Selected feature name
	selectedSession string          // Selected session ID
	width           int             // Terminal width
	height          int             // Terminal height
	err             error           // Error state
	loading         bool            // Loading state for sessions
	confirmed       bool            // Whether selection is confirmed
	// Markdown preview fields
	markdownContent      string   // Content of requirement.md
	markdownLines        []string // Rendered markdown lines
	markdownScrollOffset int      // Scroll position in markdown preview
	markdownLoading      bool     // Loading state for markdown
	// Features scroll fields
	featureScrollOffset int // Scroll position in features list
	// Input scroll fields
	inputScrollOffset int          // Scroll position in flat inputs list
	selectedInput     *UserInput   // Currently selected user input for resuming
	selectedContinue  bool         // True if continue option is selected
	latestSession     *SessionInfo // Latest session for continue functionality
}

// UserInput represents a user input in a Claude session
type UserInput struct {
	Content    string    // The user input content
	Timestamp  time.Time // When the input was made
	TurnNumber int       // Turn number in the session
}

// FlatUserInput represents a user input with session context for flat display
type FlatUserInput struct {
	UserInput
	SessionID   string    // Session ID this input belongs to
	SessionTime time.Time // Session start time for sorting
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
		flatInputs:   []FlatUserInput{},
		featureIndex: 0,
		inputIndex:   0,
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
		case "ctrl+c", "q":
			return m, tea.Quit

		case "esc":
			return m, tea.Quit

		case "enter":
			if m.activePane == 1 {
				if m.inputIndex == 0 && m.latestSession != nil {
					// Continue from latest session
					m.selectedFeature = m.features[m.featureIndex]
					m.selectedSession = m.latestSession.ID
					m.selectedInput = nil // No specific input = continue
					m.selectedContinue = true
					m.confirmed = true
					return m, tea.Quit
				} else if len(m.flatInputs) > 0 {
					// Redo from specific user input
					adjustedIndex := m.inputIndex
					if m.latestSession != nil {
						adjustedIndex = m.inputIndex - 1 // Adjust for continue block
					}
					if adjustedIndex >= 0 && adjustedIndex < len(m.flatInputs) {
						flatInput := m.flatInputs[adjustedIndex]
						m.selectedFeature = m.features[m.featureIndex]
						m.selectedSession = flatInput.SessionID
						m.selectedInput = &flatInput.UserInput
						m.selectedContinue = false
						m.confirmed = true
						return m, tea.Quit
					}
				}
			}
			return m, nil

		case "h", "left":
			// Move to left pane (features)
			if m.activePane == 1 {
				m.activePane = 0
			}
			return m, nil

		case "l", "right":
			// Move to right pane (user inputs)
			if m.activePane == 0 && (len(m.flatInputs) > 0 || m.latestSession != nil) {
				m.activePane = 1
			}
			return m, nil

		case "ctrl+u":
			if m.activePane == 0 && len(m.markdownLines) > 0 {
				// Scroll markdown preview up
				if m.markdownScrollOffset > 0 {
					m.markdownScrollOffset -= 5 // Scroll up by 5 lines
					if m.markdownScrollOffset < 0 {
						m.markdownScrollOffset = 0
					}
				}
			}
			return m, nil

		case "ctrl+d":
			if m.activePane == 0 && len(m.markdownLines) > 0 {
				// Scroll markdown preview down
				leftPaneHeight := m.height - 4 // -4 for header and footer
				markdownSectionHeight := leftPaneHeight - (leftPaneHeight * 2 / 10)
				contentHeight := markdownSectionHeight - 4 // Account for title, border, and padding
				maxScroll := len(m.markdownLines) - contentHeight
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
					m.inputIndex = 0           // Reset input selection
					m.markdownScrollOffset = 0 // Reset scroll
					m.adjustFeatureScroll()    // Adjust feature scroll to keep selection visible
					return m, tea.Batch(
						m.loadSessionsCmd(m.features[m.featureIndex]),
						m.loadMarkdownCmd(m.features[m.featureIndex]),
					)
				}
			} else if m.activePane == 1 {
				// Navigate user inputs (including continue option)
				totalItems := len(m.flatInputs)
				if m.latestSession != nil {
					totalItems++ // +1 for continue option
				}
				if m.inputIndex < totalItems-1 {
					m.inputIndex++
					m.adjustInputScroll()
				}
			}
			return m, nil

		case "k", "up":
			if m.activePane == 0 {
				// Navigate features
				if m.featureIndex > 0 {
					m.featureIndex--
					m.inputIndex = 0           // Reset input selection
					m.markdownScrollOffset = 0 // Reset scroll
					m.adjustFeatureScroll()    // Adjust feature scroll to keep selection visible
					return m, tea.Batch(
						m.loadSessionsCmd(m.features[m.featureIndex]),
						m.loadMarkdownCmd(m.features[m.featureIndex]),
					)
				}
			} else if m.activePane == 1 {
				// Navigate user inputs
				if m.inputIndex > 0 {
					m.inputIndex--
					m.adjustInputScroll()
				}
			}
			return m, nil
		}

	case sessionsLoadedMsg:
		m.sessions = msg.sessions
		m.flatInputs = msg.flatInputs
		m.latestSession = msg.latestSession
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

	// Render left pane (features + markdown preview)
	leftPane := m.renderLeftPane(paneWidth, paneHeight)

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

	// Build footer based on current state
	var footerText string
	if m.activePane == 0 {
		footerText = "j/k: navigate features • l: user inputs • ctrl+u/d: scroll preview • q: quit"
	} else {
		footerText = "j/k: navigate inputs • enter: redo from this point • h: features • q: quit"
	}

	footer := lipgloss.NewStyle().
		Foreground(lipgloss.Color("241")).
		Render(footerText)

	// Combine all parts
	return lipgloss.JoinVertical(lipgloss.Left,
		header,
		"",
		content,
		"",
		footer,
	)
}

// renderLeftPane renders the left pane with features list (top) and markdown preview (bottom)
func (m PRDResumeModel) renderLeftPane(width, height int) string {
	// Calculate heights for top and bottom sections (2:8 ratio)
	topHeight := height * 2 / 10
	bottomHeight := height - topHeight

	// Render features section (top) without bottom border
	featuresSection := m.renderFeaturesSection(width, topHeight, false)

	// Render markdown preview section (bottom) without top border
	markdownSection := m.renderMarkdownSection(width, bottomHeight, false)

	// Create a separator line
	separator := lipgloss.NewStyle().
		Width(width - 2). // -2 for border padding
		Foreground(lipgloss.Color(m.getPaneBorderColor(0))).
		Render(strings.Repeat("─", width-2))

	// Create the combined content with a unified border
	combinedContent := lipgloss.JoinVertical(lipgloss.Left, featuresSection, separator, markdownSection)

	// Apply unified border
	paneStyle := lipgloss.NewStyle().
		Width(width).
		Height(height).
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color(m.getPaneBorderColor(0)))

	return paneStyle.Render(combinedContent)
}

// renderFeaturesSection renders the features list section
func (m PRDResumeModel) renderFeaturesSection(width, topHeight int, withBorder bool) string {
	var paneStyle lipgloss.Style
	if withBorder {
		paneStyle = lipgloss.NewStyle().
			Width(width).
			Height(topHeight).
			Border(lipgloss.RoundedBorder()).
			BorderForeground(lipgloss.Color(m.getPaneBorderColor(0)))
	} else {
		paneStyle = lipgloss.NewStyle().
			Width(width - 2).     // -2 to account for parent border
			Height(topHeight - 2) // -2 to account for parent border
	}

	title := lipgloss.NewStyle().
		Bold(true).
		Foreground(lipgloss.Color("205")).
		Render("Features")

	var items []string
	items = append(items, title)
	items = append(items, "")

	// Render feature list with scrolling
	availableHeight := topHeight - 4 // Account for title and padding
	if !withBorder {
		availableHeight = topHeight - 2 // Less padding when no border
	}
	maxItems := availableHeight

	// Calculate visible range based on scroll offset
	startIdx := m.featureScrollOffset
	endIdx := startIdx + maxItems
	if endIdx > len(m.features) {
		endIdx = len(m.features)
	}

	// Display visible features
	for i := startIdx; i < endIdx; i++ {
		feature := m.features[i]
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
		for j, word := range words {
			if len(word) > 0 {
				words[j] = strings.ToUpper(word[:1]) + word[1:]
			}
		}
		displayName = strings.Join(words, " ")

		items = append(items, style.Render(prefix+displayName))
	}

	// Add scroll indicator if content is scrollable
	if len(m.features) > maxItems {
		scrollInfo := fmt.Sprintf("[%d-%d/%d]",
			startIdx+1,
			endIdx,
			len(m.features))
		indicator := lipgloss.NewStyle().
			Foreground(lipgloss.Color("241")).
			Render(scrollInfo)

		// Replace the title line to include scroll indicator
		title := lipgloss.NewStyle().
			Bold(true).
			Foreground(lipgloss.Color("205")).
			Render("Features")
		items[0] = lipgloss.JoinHorizontal(lipgloss.Left,
			title,
			" ",
			indicator)
	}

	content := strings.Join(items, "\n")
	return paneStyle.Render(content)
}

// renderMarkdownSection renders the markdown preview section
func (m PRDResumeModel) renderMarkdownSection(width, bottomHeight int, withBorder bool) string {
	var paneStyle lipgloss.Style
	if withBorder {
		paneStyle = lipgloss.NewStyle().
			Width(width).
			Height(bottomHeight).
			Border(lipgloss.RoundedBorder()).
			BorderForeground(lipgloss.Color(m.getPaneBorderColor(0)))
	} else {
		paneStyle = lipgloss.NewStyle().
			Width(width - 2).        // -2 to account for parent border
			Height(bottomHeight - 2) // -2 to account for parent border
	}

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
		contentHeight := bottomHeight - 4 // Account for title, border, and padding
		if !withBorder {
			contentHeight = bottomHeight - 2 // Less padding when no border
		}

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

// renderSessionsPane renders the right pane with flat user input blocks
func (m PRDResumeModel) renderSessionsPane(width, height int) string {
	paneStyle := lipgloss.NewStyle().
		Width(width).
		Height(height).
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color(m.getPaneBorderColor(1)))

	title := lipgloss.NewStyle().
		Bold(true).
		Foreground(lipgloss.Color("205")).
		Render("User Input History")

	var items []string
	items = append(items, title)
	items = append(items, "")

	if m.loading {
		items = append(items, "Loading user inputs...")
	} else if len(m.flatInputs) == 0 && m.latestSession == nil {
		items = append(items, lipgloss.NewStyle().
			Foreground(lipgloss.Color("241")).
			Render("No user inputs found for this feature"))
	} else {
		// Calculate visible range with scrolling
		contentHeight := height - 4 // Account for title and padding
		visibleInputs := 0
		currentHeight := 0
		totalItems := len(m.flatInputs)

		// Add continue option if we have sessions
		if m.latestSession != nil {
			totalItems++ // +1 for continue option
		}

		// Render items with scrolling support
		startIdx := m.inputScrollOffset
		itemIndex := 0

		// Render continue block if available and in visible range
		if m.latestSession != nil && startIdx == 0 {
			continueBlock := m.renderContinueBlock(width - 4) // -4 for padding
			blockLines := strings.Count(continueBlock, "\n") + 1

			if currentHeight+blockLines <= contentHeight {
				items = append(items, continueBlock)
				currentHeight += blockLines
				visibleInputs++

				// Add spacing
				if len(m.flatInputs) > 0 && currentHeight < contentHeight-1 {
					items = append(items, "")
					currentHeight++
				}
			}
			itemIndex = 1 // Continue block takes index 0
		}

		// Render user input blocks
		flatInputStartIdx := startIdx
		if m.latestSession != nil && startIdx > 0 {
			flatInputStartIdx = startIdx - 1 // Adjust for continue block
		}

		for i := flatInputStartIdx; i < len(m.flatInputs) && currentHeight < contentHeight; i++ {
			if i < 0 {
				continue
			}
			flatInput := m.flatInputs[i]
			inputBlock := m.renderUserInputBlock(flatInput, itemIndex, width-4) // -4 for padding
			blockLines := strings.Count(inputBlock, "\n") + 1

			if currentHeight+blockLines <= contentHeight {
				items = append(items, inputBlock)
				currentHeight += blockLines
				visibleInputs++

				// Add spacing between blocks
				if i < len(m.flatInputs)-1 && currentHeight < contentHeight-1 {
					items = append(items, "")
					currentHeight++
				}
			}
			itemIndex++
		}

		// Add scroll indicator if needed
		if totalItems > visibleInputs || m.inputScrollOffset > 0 {
			scrollInfo := fmt.Sprintf("[Items %d-%d/%d]",
				m.inputScrollOffset+1,
				m.inputScrollOffset+visibleInputs,
				totalItems)
			indicator := lipgloss.NewStyle().
				Foreground(lipgloss.Color("241")).
				Render(scrollInfo)

			// Update title with scroll indicator
			items[0] = lipgloss.JoinHorizontal(lipgloss.Left,
				title,
				" ",
				indicator)
		}
	}

	content := strings.Join(items, "\n")
	return paneStyle.Render(content)
}

// renderContinueBlock renders the continue option block
func (m PRDResumeModel) renderContinueBlock(width int) string {
	isSelected := m.inputIndex == 0
	isActive := m.activePane == 1

	// Determine block style based on state
	var blockStyle lipgloss.Style
	var borderColor string
	var prefix string

	if isSelected && isActive {
		borderColor = "46" // Green for continue option
		prefix = "▶ "
	} else if isSelected {
		borderColor = "250" // Light gray for inactive selection
		prefix = "▶ "
	} else {
		borderColor = "238" // Dark gray for unselected
		prefix = "  "
	}

	blockStyle = lipgloss.NewStyle().
		Width(width).
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color(borderColor)).
		Padding(0, 1)

	// Build continue block content
	headerLine := lipgloss.NewStyle().
		Bold(isSelected && isActive).
		Foreground(lipgloss.Color("46")).
		Render(fmt.Sprintf("%s[Continue Latest Session]", prefix))

	var sessionInfo string
	if m.latestSession != nil {
		timeStr := m.latestSession.StartTime.Format("2006-01-02 15:04")
		sessionIdShort := m.latestSession.ID
		if len(sessionIdShort) > 8 {
			sessionIdShort = sessionIdShort[:8]
		}
		sessionInfo = fmt.Sprintf("  %s │ Session %s │ %d turns", timeStr, sessionIdShort, m.latestSession.TurnCount)
	} else {
		sessionInfo = "  No session available"
	}

	infoLine := lipgloss.NewStyle().
		Foreground(lipgloss.Color("241")).
		Render(sessionInfo)

	descLine := lipgloss.NewStyle().
		Foreground(lipgloss.Color("250")).
		Italic(true).
		Render("  Continue from the end of the most recent session")

	var blockContent []string
	blockContent = append(blockContent, headerLine)
	blockContent = append(blockContent, infoLine)
	blockContent = append(blockContent, "")
	blockContent = append(blockContent, descLine)

	content := strings.Join(blockContent, "\n")
	return blockStyle.Render(content)
}

// renderUserInputBlock renders an individual user input block with markdown styling
func (m PRDResumeModel) renderUserInputBlock(flatInput FlatUserInput, index int, width int) string {
	isSelected := index == m.inputIndex
	isActive := m.activePane == 1

	// Determine block style based on state
	var blockStyle lipgloss.Style
	var borderColor string
	var prefix string

	if isSelected && isActive {
		borderColor = "205" // Pink for active selection
		prefix = "▶ "
	} else if isSelected {
		borderColor = "250" // Light gray for inactive selection
		prefix = "▶ "
	} else {
		borderColor = "238" // Dark gray for unselected
		prefix = "  "
	}

	blockStyle = lipgloss.NewStyle().
		Width(width).
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color(borderColor)).
		Padding(0, 1)

	// Build header with session info and timestamp
	timeStr := flatInput.SessionTime.Format("2006-01-02 15:04")
	sessionIdShort := flatInput.SessionID
	if len(sessionIdShort) > 8 {
		sessionIdShort = sessionIdShort[:8]
	}

	headerLine := lipgloss.NewStyle().
		Bold(isSelected && isActive).
		Foreground(lipgloss.Color("33")).
		Render(fmt.Sprintf("%s%s │ Session %s │ Turn %d", prefix, timeStr, sessionIdShort, flatInput.TurnNumber))

	// Render user input content as markdown
	contentLines := renderMarkdown(flatInput.Content, width-6) // -6 for padding and border

	var blockContent []string
	blockContent = append(blockContent, headerLine)
	blockContent = append(blockContent, "")

	// Add content lines with proper styling
	blockContent = append(blockContent, contentLines...)
	content := strings.Join(blockContent, "\n")
	return blockStyle.Render(content)
}

// adjustFeatureScroll ensures the selected feature is visible in the features section
func (m *PRDResumeModel) adjustFeatureScroll() {
	// Calculate available height for features section (same logic as renderFeaturesSection)
	leftPaneHeight := m.height - 4 // -4 for header and footer
	topHeight := leftPaneHeight * 2 / 10
	availableHeight := topHeight - 4 // Account for title and padding
	maxItems := availableHeight      // This is the number of visible items
	// Adjust scroll offset to keep selected feature visible
	if m.featureIndex < m.featureScrollOffset {
		// Selected item is above visible area, scroll up
		m.featureScrollOffset = m.featureIndex
	} else if m.featureIndex >= m.featureScrollOffset+maxItems {
		// Selected item is below visible area, scroll down
		m.featureScrollOffset = m.featureIndex - maxItems + 1
	}

	// Ensure scroll offset doesn't go negative
	if m.featureScrollOffset < 0 {
		m.featureScrollOffset = 0
	}

	// Ensure scroll offset doesn't exceed reasonable bounds
	maxScroll := len(m.features) - maxItems
	if maxScroll < 0 {
		maxScroll = 0
	}
	if m.featureScrollOffset > maxScroll {
		m.featureScrollOffset = maxScroll
	}
}

// adjustInputScroll ensures the selected input is visible in the flat inputs list
func (m *PRDResumeModel) adjustInputScroll() {
	totalItems := len(m.flatInputs)
	if m.latestSession != nil {
		totalItems++ // +1 for continue option
	}

	if totalItems == 0 {
		return
	}

	// Calculate actual visible items using the same logic as renderSessionsPane
	paneWidth := (m.width - 3) / 2      // Same calculation as renderSessionsPane
	contentHeight := (m.height - 4) - 4 // Account for pane header, footer, title and padding
	visibleItems := m.calculateVisibleInputs(paneWidth, contentHeight)

	if visibleItems <= 0 {
		return // Not enough space to show items
	}

	// Adjust scroll offset to keep selected input visible
	if m.inputIndex < m.inputScrollOffset {
		// Selected item is above visible area, scroll up
		m.inputScrollOffset = m.inputIndex
	} else if m.inputIndex >= m.inputScrollOffset+visibleItems {
		// Selected item is below visible area, scroll down
		m.inputScrollOffset = m.inputIndex - visibleItems + 1
	}

	// Ensure scroll offset doesn't go negative
	if m.inputScrollOffset < 0 {
		m.inputScrollOffset = 0
	}

	// Ensure scroll offset doesn't exceed reasonable bounds
	maxScroll := totalItems - visibleItems
	if maxScroll < 0 {
		maxScroll = 0
	}
	if m.inputScrollOffset > maxScroll {
		m.inputScrollOffset = maxScroll
	}
}

// calculateVisibleInputs dynamically calculates how many input blocks can fit in the visible area
func (m *PRDResumeModel) calculateVisibleInputs(width, contentHeight int) int {
	if len(m.flatInputs) == 0 || contentHeight <= 0 {
		return 0
	}

	visibleItems := 0
	currentHeight := 0

	// Calculate starting from current scroll offset to simulate actual rendering
	startIdx := m.inputScrollOffset
	for i := startIdx; i < len(m.flatInputs) && currentHeight < contentHeight; i++ {
		flatInput := m.flatInputs[i]

		// Calculate block height the same way as renderUserInputBlock
		inputBlock := m.renderUserInputBlock(flatInput, i, width-4) // -4 for padding
		blockLines := strings.Count(inputBlock, "\n") + 1

		if currentHeight+blockLines <= contentHeight {
			visibleItems++
			currentHeight += blockLines

			// Add spacing between blocks (same as renderSessionsPane)
			if i < len(m.flatInputs)-1 && currentHeight < contentHeight-1 {
				currentHeight++
			}
		} else {
			break
		}
	}

	return visibleItems
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
		sessions, flatInputs, latestSession, err := m.loadSessionsForFeature(feature)
		if err != nil {
			return errorMsg{err: err}
		}
		return sessionsLoadedMsg{sessions: sessions, flatInputs: flatInputs, latestSession: latestSession}
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

// loadSessionsForFeature loads Claude sessions related to a feature and creates flat input list
func (m PRDResumeModel) loadSessionsForFeature(feature string) ([]SessionInfo, []FlatUserInput, *SessionInfo, error) {
	// Create feature session state manager
	featureDir := filepath.Join(".kiro", "spec", feature)
	sessionManager := claude.NewFeatureSessionStateManager(featureDir)

	// Load all sessions from sessions.json
	sessionsState, err := sessionManager.LoadSessions()
	if err != nil {
		return nil, nil, nil, fmt.Errorf("failed to load sessions: %w", err)
	}

	var sessions []SessionInfo
	var flatInputs []FlatUserInput

	for _, state := range sessionsState {
		// Parse the transcript file to get session details
		sessionInfo, err := m.parseTranscriptFile(state.TranscriptPath, state)
		if err != nil {
			// Skip files that can't be parsed
			continue
		}

		if sessionInfo != nil {
			sessions = append(sessions, *sessionInfo)

			// Create flat user inputs from this session
			for _, userInput := range sessionInfo.UserInputs {
				flatInputs = append(flatInputs, FlatUserInput{
					UserInput:   userInput,
					SessionID:   sessionInfo.ID,
					SessionTime: sessionInfo.StartTime,
				})
			}
		}
	}

	// Sort sessions by start time (newest first)
	sort.Slice(sessions, func(i, j int) bool {
		return sessions[i].StartTime.After(sessions[j].StartTime)
	})

	// Sort flat inputs by timestamp (newest first), then by turn number within same session
	sort.Slice(flatInputs, func(i, j int) bool {
		if flatInputs[i].SessionTime.Equal(flatInputs[j].SessionTime) {
			return flatInputs[i].TurnNumber > flatInputs[j].TurnNumber
		}
		return flatInputs[i].SessionTime.After(flatInputs[j].SessionTime)
	})

	// Find latest session for continue functionality
	var latestSession *SessionInfo
	if len(sessions) > 0 {
		latestSession = &sessions[0] // First session is newest due to sorting
	}

	return sessions, flatInputs, latestSession, nil
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

							currentTurn++

							// Add to user inputs list
							userInputs = append(userInputs, UserInput{
								Content:    content,
								Timestamp:  state.StartedAt, // Use session start time as approximation
								TurnNumber: currentTurn,
							})

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
	sessions      []SessionInfo
	flatInputs    []FlatUserInput
	latestSession *SessionInfo
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

// GetSelectedInput returns the selected user input if any
func (m PRDResumeModel) GetSelectedInput() *UserInput {
	if m.confirmed {
		return m.selectedInput
	}
	return nil
}

// GetSelectedContinue returns true if continue option was selected
func (m PRDResumeModel) GetSelectedContinue() bool {
	if m.confirmed {
		return m.selectedContinue
	}
	return false
}

// RunPRDResume runs the PRD resume UI and returns the selected feature, session, optional input, and continue flag
func RunPRDResume(features []string) (string, string, *UserInput, bool, error) {
	model := NewPRDResumeModel(features)
	p := tea.NewProgram(model, tea.WithAltScreen())

	finalModel, err := p.Run()
	if err != nil {
		return "", "", nil, false, fmt.Errorf("failed to run PRD resume UI: %w", err)
	}

	if m, ok := finalModel.(PRDResumeModel); ok {
		return m.GetSelectedFeature(), m.GetSelectedSession(), m.GetSelectedInput(), m.GetSelectedContinue(), nil
	}

	return "", "", nil, false, fmt.Errorf("unexpected model type")
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
