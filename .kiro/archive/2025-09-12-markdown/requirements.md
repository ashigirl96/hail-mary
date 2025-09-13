# Requirements

## Overview
Implement proper markdown code block nesting rules in the hail-mary project to ensure correct rendering of nested code blocks in markdown documentation.

## Purpose
When writing documentation with nested code blocks in markdown, the parent blocks must use 4 backticks (````) and child blocks must use 3 backticks (```) to ensure proper rendering in markdown preview. This is essential for maintaining readable documentation, especially in slash command documentation files.

## User Stories
- As a developer, I want markdown files with nested code blocks to render correctly so that documentation is readable and properly formatted
- As a maintainer, I want consistent markdown formatting rules so that all documentation follows the same standard
- As a Claude Code user, I want slash command documentation to display correctly so that I can understand how to use commands

## Acceptance Criteria
- All markdown files containing nested code blocks must use 4 backticks for parent blocks
- Inner code blocks must continue to use 3 backticks
- Existing documentation files must be updated to follow this rule
- The fix must not break any existing functionality or tests

## Technical Requirements
- Update `.claude/commands/hm/steering.md` to use proper nesting
- Ensure embedded resources are correctly formatted
- Maintain backward compatibility with existing markdown files
- Follow the pattern: Parent blocks (````) → Child blocks (```)

## Priority

P0: Fix existing markdown nesting issues in slash command documentation
P1: Update any template generation to follow the nesting rule
P2: Add validation or linting for future markdown files
P3: Create comprehensive documentation about the nesting rule

## Risk and Mitigation
- Risk: Breaking existing documentation rendering
  - Mitigation: Test all changes before committing
- Risk: Embedded resources might not update correctly
  - Mitigation: Verify that embedded files are properly included and updated
