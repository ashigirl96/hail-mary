# Kiro Specification Context

You are working on a Kiro project specification. Your task is to implement the requirements defined in the specification files below.

## Current Specification

Name: {spec_name}
Path: {path_str}

## Specification Files

<kiro_spec_name>{spec_name}</kiro_spec_name>
<kiro_spec_path>{path_str}</kiro_spec_path>
<kiro_requirements_path>{path_str}/requirements.md</kiro_requirements_path>
<kiro_design_path>{path_str}/design.md</kiro_design_path>
<kiro_tasks_path>{path_str}/tasks.md</kiro_tasks_path>
<kiro_memo_path>{path_str}/memo.md</kiro_memo_path>
<kiro_investigation_path>{path_str}/investigation.md</kiro_investigation_path>

## File Descriptions

- **requirements.md**: Comprehensive requirements including user stories, acceptance criteria, and functional requirements
- **design.md**: Technical design with architecture decisions and implementation approach
- **tasks.md**: Implementation tasks with priorities and dependencies
- **memo.md**: Additional notes and context from the user
- **investigation.md**: Research findings, key discoveries, and technical considerations from investigation phase

## Instructions

1. Read the requirements in <kiro_requirements_path/> to understand what needs to be built
2. Review investigation findings in <kiro_investigation_path/> for research insights
3. Follow the technical approach in <kiro_design_path/>
4. Track your progress against tasks in <kiro_tasks_path/>

When you need to reference these files, use the XML tag paths provided above.

## RULES

- **DO NOT read memo.md**: The memo.md file contains internal developer notes and implementation details that are not part of the formal specification. You must NOT read or access {path_str}/memo.md under any circumstances.