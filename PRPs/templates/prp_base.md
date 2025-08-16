# PRP: [Feature Name]

## Goal

**Feature Goal**: [One sentence describing what this enables]

**Deliverable**: [Concrete, measurable output]

**Success Definition**: [How we know it works]

## Context

```yaml
references:
  documentation:
    - name: "[Doc Name]"
      url: "[Specific URL with #anchor]"
      purpose: "[Why this doc matters]"
    
  codebase_patterns:
    - file: "[path/to/file.ext:line]"
      pattern: "[Pattern name]"
      usage: "[How to apply this pattern]"
    
  external_examples:
    - source: "[GitHub/StackOverflow/Blog]"
      url: "[Direct link]"
      relevance: "[What to extract from this]"
    
  gotchas:
    - issue: "[Common problem]"
      solution: "[Specific fix]"
      reference: "[Where this was discovered]"

project_specifics:
  naming_conventions:
    - type: "[Component type]"
      pattern: "[Exact naming pattern]"
      example: "[Concrete example]"
  
  file_locations:
    - purpose: "[What goes here]"
      path: "[Exact path]"
      existing_examples: "[Similar files to reference]"
  
  dependencies:
    - name: "[Library/tool]"
      version: "[Specific version]"
      usage: "[How it's used in this context]"

validation_commands:
  lint: "[Exact command]"
  test: "[Exact command]"
  build: "[Exact command]"
  verify: "[Project-specific verification]"
```

## Implementation Tasks

### Task 1: [Specific Task Name]
**Objective**: [What this accomplishes]
**Location**: `[exact/path/to/file.ext]`
**Pattern Reference**: [Which pattern from context to follow]
**Key Implementation Points**:
- [Specific detail with exact naming]
- [Configuration requirement with values]
- [Integration point with other components]

**Validation Gate**: 
```bash
[Exact command to verify this task]
```

### Task 2: [Specific Task Name]
**Dependencies**: Task 1
**Objective**: [What this accomplishes]
**Location**: `[exact/path/to/file.ext]`
**Pattern Reference**: [Which pattern from context to follow]
**Key Implementation Points**:
- [Specific detail with exact naming]
- [Configuration requirement with values]
- [Integration point with other components]

**Validation Gate**: 
```bash
[Exact command to verify this task]
```

### Task 3: [Integration Task]
**Dependencies**: Tasks 1, 2
**Objective**: [How components connect]
**Locations**: 
- `[path/to/integration/point1.ext]`
- `[path/to/integration/point2.ext]`
**Key Integration Points**:
- [Exact import/export requirements]
- [Configuration updates needed]
- [Registration or initialization steps]

**Validation Gate**: 
```bash
[Exact command to verify integration]
```

## Error Handling

### Expected Errors
```yaml
- error: "[Exact error message]"
  cause: "[Why this happens]"
  solution: "[Exact fix]"
  
- error: "[Exact error message]"
  cause: "[Why this happens]"
  solution: "[Exact fix]"
```

## Testing Strategy

### Unit Tests
**Location**: `[exact/test/path]`
**Pattern**: [Reference to existing test pattern]
**Coverage Requirements**:
- [Specific function/component to test]
- [Edge case to cover]
- [Integration point to verify]

### Integration Tests
**Location**: `[exact/test/path]`
**Setup Requirements**: [Any special setup needed]
**Key Scenarios**:
- [Scenario 1 with expected outcome]
- [Scenario 2 with expected outcome]

## Final Validation Checklist

### Functional Requirements
- [ ] [Specific feature works as described]
- [ ] [Integration point A connects properly]
- [ ] [Integration point B connects properly]
- [ ] [Error handling works for case X]

### Code Quality
- [ ] All tests pass: `[exact command]`
- [ ] Linting passes: `[exact command]`
- [ ] Build succeeds: `[exact command]`
- [ ] Type checking passes: `[exact command]`

### Project Conventions
- [ ] Follows naming pattern: [specific pattern]
- [ ] Located in correct directory: [path]
- [ ] Includes required documentation: [what docs]
- [ ] Permissions configured: [where/how]

## Additional Notes

[Any project-specific context that doesn't fit above categories]

---

**PRP Confidence Score**: [X/10]
**Estimated Implementation Time**: [X hours]
**Risk Areas**: [Any areas of uncertainty]