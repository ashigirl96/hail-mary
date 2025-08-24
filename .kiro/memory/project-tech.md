## Hail-Mary Memory MCP Service - Clean Architecture Implementation
*Tags: clean-architecture, rust, mcp, repository-pattern, hail-mary, memory-service*
*References: 4, Confidence: 0.95*

The MemoryMcpServer in src/services/memory_mcp.rs implements the Interface Adapter layer in clean architecture. Key design patterns:

1. **Repository Pattern**: Generic over R: MemoryRepository trait, allowing both SqliteMemoryRepository (production) and InMemoryRepository (testing)

2. **Data Transformation Layer**: Converts between MCP protocol types (MemoryInputMcp) and domain types (MemoryInput) with validation

3. **Error Boundary**: Maps domain errors (MemoryError) to protocol errors (McpError) with proper error codes

4. **Dependency Inversion**: Service depends on repository trait, not concrete implementation

5. **Concurrency**: Uses Arc<Mutex<MemoryService<R>>> for thread-safe async access

6. **Layer Structure**:
   - MCP Protocol Layer (External)
   - MemoryMcpServer (Interface Adapter)
   - MemoryService (Use Case/Service Layer)  
   - MemoryRepository (Interface)
   - SqliteMemoryRepository (Infrastructure)

---

## Hail-Mary MCP Tool Implementation - Remember and Recall
*Tags: mcp-tools, rust, validation, hail-mary, remember, recall*
*References: 12, Confidence: 0.90*

The Memory MCP service exposes two main tools via rmcp framework:

1. **remember tool**: 
   - Accepts RememberParams with array of MemoryInputMcp
   - Validates memory types against KiroConfig
   - Batch stores memories via service.remember_batch()
   - Returns RememberResponse with memory_ids and created_count

2. **recall tool**:
   - Accepts RecallParams with query, optional type/tags/limit filters
   - Validates type filter against allowed types in config
   - Uses service.recall() with FTS5 search
   - Returns RecallResponse with markdown-formatted content

3. **Validation Strategy**:
   - Input validation at MCP layer (type checking against config)
   - Business validation at service layer (empty title, content)
   - Proper error propagation with MCP error codes (INVALID_PARAMS, INTERNAL_ERROR)

---

