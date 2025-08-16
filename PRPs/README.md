# PRP (Prompt Reference Package) System

## What is a PRP?

A **Prompt Reference Package (PRP)** is a comprehensive, context-rich document that enables AI agents to successfully implement features in a single pass. It contains all necessary context, patterns, and references that an AI would need to complete a task without requiring multiple rounds of clarification or exploration.

## Core Philosophy

**One-Pass Implementation Success**: A well-crafted PRP should enable an AI agent with no prior knowledge of the codebase to successfully implement the feature using only:
- The PRP content itself
- Access to codebase files (with guidance on which ones)
- Its training data knowledge

## PRP Structure

Each PRP follows a standardized template with these sections:

1. **Goal Definition**: Clear, measurable objectives
2. **Context Curation**: YAML-structured references to documentation, patterns, and gotchas
3. **Implementation Tasks**: Dependency-ordered, information-dense task specifications
4. **Validation Gates**: Project-specific commands to verify success
5. **Final Checklist**: Comprehensive validation criteria

## Quality Standards

### Information Density
- Every reference must be **specific and actionable**
- URLs include section anchors (#specific-section)
- File references include exact patterns to follow
- Task specifications use project-specific naming conventions

### Context Completeness
Apply the "No Prior Knowledge" test:
> "If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"

### Template Compliance
- All required sections must be completed
- Tasks follow dependency ordering
- Validation commands are verified working

## PRP Types

### Base PRP
Standard feature implementation with research phase. Used for:
- New features requiring exploration
- Complex integrations
- Multi-component changes

### Quick PRP
Focused implementation without research phase. Used for:
- Bug fixes with known solutions
- Small, well-defined changes
- Pattern-based implementations

## Directory Structure

```
PRPs/
├── README.md              # This file
├── templates/            
│   ├── prp_base.md       # Standard template
│   └── prp_quick.md      # Quick implementation template
├── ai_docs/              # Critical external documentation
│   └── *.md              # Curated docs for complex features
└── *.md                  # Generated PRPs
```

## Creating a PRP

### Research Phase
1. **Codebase Analysis**: Search for similar patterns, conventions, test approaches
2. **External Research**: Library docs, implementation examples, best practices
3. **User Clarification**: Ask for missing requirements

### Generation Phase
1. Choose appropriate template
2. Apply "No Prior Knowledge" test
3. Transform research into template sections
4. Ensure information density standards
5. Validate all references are accessible

## Using a PRP

When implementing from a PRP:
1. Read the entire PRP first
2. Follow the task order exactly
3. Use specified validation gates after each major step
4. Complete the final checklist before considering done

## Success Metrics

A successful PRP achieves:
- **Confidence Score**: 8-10/10 for one-pass implementation
- **Zero Clarifications**: No need to ask for additional context
- **First-Try Success**: Implementation works without major revisions

## Best Practices

### DO
- Include specific file:line references
- Provide exact command strings
- Reference actual code patterns in the codebase
- Include common error messages and solutions
- Specify exact naming conventions

### DON'T
- Use generic phrases like "follow best practices"
- Reference files without explaining their purpose
- Assume knowledge of project conventions
- Skip validation steps
- Leave ambiguous requirements

## Maintenance

PRPs should be updated when:
- Major architectural changes occur
- New patterns are established
- Dependencies are upgraded
- Implementation reveals missing context

Mark outdated PRPs with a header warning:
```markdown
> ⚠️ **OUTDATED**: This PRP was created for version X.Y.Z and may not reflect current patterns
```