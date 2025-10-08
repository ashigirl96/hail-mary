# Memo: enhance-investigate

## Examples

### Example 1: New Topic Investigation
```
/spec:investigate

> 🔍 What would you like to investigate?
> [Provide specific technical question or area]

[STOP AND WAIT]

User: "JWT authentication implementation"

> 🚀 Investigation Plan for "JWT Authentication":
> [Parallel agents launch...]

> 📝 Investigation saved to <kiro_investigation_path>
> Topic: "JWT Authentication" (Section #1)
> Confidence: High (90%)

> 🔄 Continue investigating "JWT Authentication"?

User: Y, what about refresh token rotation?

> 📝 Updated investigation for "JWT Authentication"
> Added findings about refresh token rotation
> Confidence: High (92%)

> 🔄 Continue investigating "JWT Authentication"?

User: done

> ✅ Investigation complete
> • Topics investigated: 1
> • Overall confidence: High (92%)
```

### Example 2: Resume Existing Topic
```
/spec:investigate --topic "JWT Authentication" --for requirements

> 📝 Resuming existing topic "JWT Authentication" from investigation.md
> Previous confidence: High (92%)

> 🚀 Continuing investigation for "JWT Authentication":
> [Parallel agents focus on gaps/updates...]

> 📝 Updated investigation for "JWT Authentication"
> Topic: "JWT Authentication" (Section #1 - Updated)
> Confidence: High (95%)

> 🔄 Continue investigating "JWT Authentication"?

User: done

> ✅ Investigation complete
> • Topics investigated: 1 (resumed)
> • Overall confidence: High (95%)
> • Updated: requirements.md (Technical Requirements section)
```

### Example 3: Multiple Separate Investigations
```
# First investigation
/spec:investigate

> 🔍 What would you like to investigate?

User: "API rate limiting"

[Investigation process...]

> 🔄 Continue investigating "API Rate Limiting"?

User: done

> ✅ Investigation complete
> • Topics investigated: 1
> • Overall confidence: High (85%)

# Second investigation (new command)
/spec:investigate

> 🔍 What would you like to investigate?

User: "caching strategy"

> 🚀 Investigation Plan for "Caching Strategy":
> [New parallel investigation...]

> 📝 Investigation saved to <kiro_investigation_path>
> Topic: "Caching Strategy" (Section #2)
> Confidence: Medium (75%)

> 🔄 Continue investigating "Caching Strategy"?

User: done

> ✅ Investigation complete
> • Topics investigated: 1
> • Overall confidence: Medium (75%)

# Third investigation (resuming first topic)
/spec:investigate --topic "API Rate Limiting"

> 📝 Resuming existing topic "API Rate Limiting" from investigation.md
> Previous confidence: High (85%)

[Continue investigation with new angle...]
```

### Example 4: Investigation Guided by Steering
````
/spec:investigate

> 🔍 What would you like to investigate?

User: "How to query production database for users table id=123 name field?"

> 🚀 Investigation Plan for "Production Database Query":
>
> Using steering guidance from `bigquery` type:
> - Focus on EXTERNAL_QUERY patterns for Cloud SQL
> - Check security best practices
> - Look for performance optimization patterns
>
> [Parallel agents launch with steering-informed focus...]

> 📝 Investigation saved to investigation.md
> Topic: "Production Database Query" (Section #1)
> Confidence: High (92%)
> **Guided by**: steering:bigquery (EXTERNAL_QUERY patterns)
>
> ### Summary
> Found optimal approach using BigQuery EXTERNAL_QUERY for Cloud SQL production access
>
> ### Evidence
> **From Codebase (src/services/data_access.ts:89)**:
> ```typescript
> // Existing pattern for production queries
> const query = `SELECT name FROM
>   EXTERNAL_QUERY('projects/prod-project/locations/asia-northeast1/connections/prod-mysql',
>   'SELECT name FROM users WHERE id = 123')`;
> ```
>
> **From Context7 (BigQuery documentation)**:
> - EXTERNAL_QUERY syntax for Cloud SQL connections
> - Security: Always use read-only credentials for production
> - Performance: Batch queries to minimize round-trips
>
> ### Recommendations
> 1. Use EXTERNAL_QUERY pattern from BigQuery to access production Cloud SQL
> 2. Ensure read-only access credentials are configured
> 3. Follow existing implementation pattern in data_access.ts
>
> ### Investigation Notes
> - **Steering Applied**: Used `bigquery` type patterns to identify EXTERNAL_QUERY as the appropriate solution

> 🔄 Continue investigating "Production Database Query"?

User: done

> ✅ Investigation complete
````


---------


### Investigation Structure
````markdown
# Investigation - [Spec Name]

## Topic: [Auto-generated Topic Title]
**Confidence**: [level] ([percentage]%)
**Primary Sources**: src/path/* ([%]), Context7:lib ([%]), web ([%])
**Guided by**: steering:[type-name] (patterns and criteria)

### Summary
[1-2 line executive summary of findings]

### Root Cause / Core Finding
[Main discovery - flexible format based on content type]
- Architecture diagrams (mermaid)
- Code implementations
- System designs
- Data flows

### Evidence
[Source-based evidence with attribution]

**From Codebase (path/file.ts:lines)**:
```language
// Actual implementation code
```

**From Context7 (library docs)**:
- [Official patterns and best practices]

**From Web (as last resort)**:
- [Recent developments or community solutions]

### Recommendations
1. [Actionable recommendation]
2. [Implementation approach]
3. [Consideration or trade-off]

### Investigation Notes
- **Update [time]**: [Additional findings or corrections]
- **Correction**: [Fixed understanding or updated information]
- **Note**: [Important observations or caveats]
- **Steering Applied**: Used [type-name] patterns to guide investigation focus
````

