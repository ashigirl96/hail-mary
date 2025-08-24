# Effective System Prompt Design Guide

*A comprehensive guide for creating high-performance system prompts based on Claude Code analysis*

---

## 🚀 Quick Reference

### The Golden Rules
1. **Identity First**: Start with "You are..." to establish role
2. **Show, Don't Tell**: 30-40% should be examples
3. **Progressive Disclosure**: Simple → Complex information flow
4. **Cooperative Language**: "I'll help..." instead of "FORBIDDEN"
5. **Graceful Boundaries**: Guide through expertise, not walls

### Optimal Structure
```
1. Identity & Purpose (5-10%)
2. Tone & Style (10-15%)
3. Core Principles (15-20%)
4. Examples with Reasoning (30-40%)
5. Tool Usage & Workflow (20-25%)
6. Edge Cases & Recovery (10-15%)
```

---

## 📊 Design Layers Model

### Layer 1: Surface (Syntax & Format)
- **Language Style**: Second person "You should/must" vs imperative
- **Structure**: Hierarchical sections with clear headers
- **Formatting**: Use of tags (`<example>`, `<reasoning>`)
- **Visual Markers**: IMPORTANT:, Note:, Warning: for emphasis

### Layer 2: Architecture (Design Patterns)
- **Information Flow**: Progressive elaboration
- **Error Handling**: Graceful degradation vs fail-safe
- **State Management**: Stateful conversation vs stateless execution
- **Context Awareness**: Adaptive behavior vs rigid rules

### Layer 3: Philosophy (Core Beliefs)
- **Agency Model**: Partner vs Tool
- **Learning Approach**: Example-based vs Rule-based
- **Relationship**: Collaborative vs Authoritative
- **Adaptation**: Context-aware vs Context-free

---

## 🎯 Implementation Patterns

### Pattern 1: Identity