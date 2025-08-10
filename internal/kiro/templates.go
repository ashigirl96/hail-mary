package kiro

import (
	"bytes"
	_ "embed"
	"fmt"
	"text/template"
	"time"
)

// Embed the requirements template at build time
//
//go:embed templates/requirements.md
var requirementsTemplateFile string

// TemplateData holds the data for rendering the requirements template
type TemplateData struct {
	RequirementsPath string
}

// GetRequirementsTemplate returns the rendered requirements template
// Note: This now returns the PRD system prompt with the requirements path embedded
func GetRequirementsTemplate(requirementsPath string) string {
	// Parse the embedded template
	tmpl, err := template.New("requirements").Parse(requirementsTemplateFile)
	if err != nil {
		// Fallback to a simple template if parsing fails
		return getFallbackTemplate(requirementsPath)
	}

	// Prepare the data
	data := TemplateData{
		RequirementsPath: requirementsPath,
	}

	// Execute the template
	var buf bytes.Buffer
	if err := tmpl.Execute(&buf, data); err != nil {
		// Fallback if execution fails
		return getFallbackTemplate(requirementsPath)
	}

	return buf.String()
}

// getFallbackTemplate returns a simple fallback template if the main template fails
func getFallbackTemplate(requirementsPath string) string {
	return "# Product Requirements Document Specialist\n\n" +
		"Your documentation environment is configured as:\n" +
		"- **Output location**: " + requirementsPath + "\n\n" +
		"You are a requirements documentation specialist who helps users create clear, minimal, and actionable requirements documentation in EARS format.\n"
}

// GetInitialRequirementsContent returns the initial content for a new requirements.md file
func GetInitialRequirementsContent(featureTitle string) string {
	timestamp := time.Now().Format("2006-01-02")

	return fmt.Sprintf(`# 要件定義書: %s

## 概要
- Date: %s

## 問題定義
[この機能が解決する問題を記述]

## 要件
(要件は機能の説明後に記載されます)
`, featureTitle, timestamp)
}
