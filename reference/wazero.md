# Wazero: Zero-Dependency WebAssembly Runtime for Go

## Executive Summary

**wazero** is a WebAssembly runtime written in pure Go with zero dependencies, developed by Tetrate Labs. Released as v1.0 in March 2023, it provides a unique solution for Go developers who need WebAssembly capabilities without the complexity of CGO dependencies.

## Table of Contents
- [Core Architecture](#core-architecture)
- [Performance Analysis](#performance-analysis)
- [API Design](#api-design)
- [Production Use Cases](#production-use-cases)
- [Comparison with Alternatives](#comparison-with-alternatives)
- [CGO and Its Implications](#cgo-and-its-implications)
- [Implementation Examples](#implementation-examples)
- [Best Practices](#best-practices)
- [Future Roadmap](#future-roadmap)

## Core Architecture

### Design Philosophy

wazero follows a minimalist philosophy centered around three core principles:

1. **Zero Dependencies**: No external libraries, no CGO, no shared libraries
2. **Go-Native Integration**: Leverages Go's strengths (concurrency, context, error handling)
3. **Standards Compliance**: Full WebAssembly Core Specification 1.0 and 2.0 compliance

### Runtime Modes

wazero offers two distinct runtime modes:

#### Interpreter Mode
- Platform-independent implementation
- No architecture-specific code
- Slower execution but maximum portability
- Useful for development and debugging

#### Compiler Mode (Default)
- Ahead-of-Time (AOT) compilation to native machine code
- 10x+ performance improvement over interpreter
- Platform-specific optimizations
- Memory-mapped executable code

```go
// Default: Compiler mode
runtime := wazero.NewRuntime(ctx)

// Explicit: Interpreter mode
runtime := wazero.NewRuntimeWithConfig(ctx, 
    wazero.NewRuntimeConfigInterpreter())
```

### Component Architecture

```
┌─────────────────────────────────────┐
│          Go Application             │
├─────────────────────────────────────┤
│           wazero Runtime            │
│  ┌─────────────────────────────┐   │
│  │     Compilation Phase       │   │
│  │  WASM Binary → CompiledModule│   │
│  └─────────────────────────────┘   │
│  ┌─────────────────────────────┐   │
│  │    Instantiation Phase      │   │
│  │  CompiledModule → Module    │   │
│  └─────────────────────────────┘   │
│  ┌─────────────────────────────┐   │
│  │      Execution Phase        │   │
│  │   Module.Call() → Results   │   │
│  └─────────────────────────────┘   │
└─────────────────────────────────────┘
```

### Memory Management

- **Sandboxed Execution**: Each module runs in isolation
- **Linear Memory**: WebAssembly standard memory model
- **Garbage Collection**: Leverages Go's GC for host-side memory
- **Resource Limits**: Configurable memory limits per module

## Performance Analysis

### Benchmark Comparison (2023)

Based on comprehensive benchmarks comparing WebAssembly runtimes:

| Runtime | Backend | Relative Performance | Compilation Speed | Binary Size |
|---------|---------|---------------------|------------------|-------------|
| Wasmtime | Cranelift | 1.0x (baseline) | Slow | Large (CGO) |
| Wasmer | Cranelift | ~1.0x | Slow | Large (CGO) |
| Wasmer | Singlepass | ~1.2x | Fast | Large (CGO) |
| wazero | Compiler | ~2-3x slower | Fast | Small (Pure Go) |
| wazero | Interpreter | ~10-20x slower | Instant | Small (Pure Go) |

### Real-World Performance

Despite slower synthetic benchmarks, wazero achieves excellent real-world performance:

#### Arcjet Case Study
- **Use Case**: API request processing with WebAssembly
- **Performance Targets**: p50 < 10ms, p99 < 30ms
- **Optimization Techniques**:
  - Pre-compilation with Wizer
  - wasm-opt optimizations
  - Module caching
- **Result**: Successfully met all latency targets

#### DoltHub Case Study
- **Use Case**: MySQL regex compatibility layer
- **Challenge**: Implement MySQL's regex engine in Go
- **Solution**: Compile MySQL regex to WASM, run with wazero
- **Performance Impact**: Only 15% slowdown for complete compatibility
- **Benefit**: Full MySQL regex features including backreferences

### Performance Optimization Strategies

```go
// 1. Pre-compile modules for reuse
compiled, _ := runtime.CompileModule(ctx, wasmBytes)

// 2. Cache compiled modules
var moduleCache = make(map[string]wazero.CompiledModule)

// 3. Use module configuration for optimization
config := wazero.NewModuleConfig().
    WithStartFunctions().  // Control start function execution
    WithName("optimized")  // Named modules for debugging

// 4. Batch operations when possible
module, _ := runtime.InstantiateModule(ctx, compiled, config)
```

## API Design

### Core API Components

#### Runtime
The top-level container managing compilation and execution:

```go
// Create runtime with context
ctx := context.Background()
r := wazero.NewRuntime(ctx)
defer r.Close(ctx) // Always close to free resources

// With custom configuration
config := wazero.NewRuntimeConfig().
    WithCoreFeatures(api.CoreFeaturesV2).
    WithMemoryLimitPages(256) // 16MB limit

r := wazero.NewRuntimeWithConfig(ctx, config)
```

#### Module Management

```go
// Compile once, instantiate many times
compiled, err := r.CompileModule(ctx, wasmBinary)
if err != nil {
    return fmt.Errorf("compilation failed: %w", err)
}

// Create multiple isolated instances
for i := 0; i < 10; i++ {
    module, err := r.InstantiateModule(ctx, compiled,
        wazero.NewModuleConfig().
            WithName(fmt.Sprintf("worker_%d", i)))
    
    // Each module is isolated
    go processWithModule(module)
}
```

### Host Functions

Host functions enable WebAssembly to call Go functions:

```go
// Define a host function
func hostLog(ctx context.Context, m api.Module, offset, length uint32) {
    // Read string from module memory
    buf, ok := m.Memory().Read(offset, length)
    if !ok {
        return
    }
    fmt.Printf("WASM says: %s\n", buf)
}

// Register host module
_, err := r.NewHostModuleBuilder("env").
    NewFunctionBuilder().
    WithParameterNames("offset", "length").
    WithFunc(hostLog).
    Export("log").
    Instantiate(ctx)
```

### Advanced Host Function Patterns

```go
// Closure-based host functions with state
type Logger struct {
    prefix string
    mu     sync.Mutex
}

func (l *Logger) CreateHostFunction() interface{} {
    return func(ctx context.Context, msg string) {
        l.mu.Lock()
        defer l.mu.Unlock()
        log.Printf("%s: %s", l.prefix, msg)
    }
}

// Context propagation
func timedFunction(ctx context.Context, m api.Module) uint32 {
    // Respect context deadline
    select {
    case <-ctx.Done():
        return 0 // Timeout
    default:
        // Perform operation
        return 1
    }
}
```

### WASI Implementation

wazero includes built-in WASI (WebAssembly System Interface) support:

```go
// Configure WASI
wasi := r.NewHostModuleBuilder("wasi_snapshot_preview1")

// File system access
config := wazero.NewModuleConfig().
    WithFS(os.DirFS("/allowed/path")).
    WithStdout(os.Stdout).
    WithStderr(os.Stderr).
    WithStdin(os.Stdin).
    WithArgs("arg1", "arg2").
    WithEnv("KEY", "value")

// Instantiate with WASI
module, err := r.InstantiateModule(ctx, wasmBinary, config)
```

### Memory Management

```go
// Direct memory access
memory := module.Memory()

// Write data
data := []byte("Hello, WASM!")
ok := memory.Write(0, data)

// Read data
buffer, ok := memory.Read(0, uint32(len(data)))

// Get memory size
size := memory.Size()

// Grow memory (in pages, 1 page = 64KB)
newSize, ok := memory.Grow(10)
```

## Production Use Cases

### 1. Plugin Systems

#### go-plugin Framework
A production-ready plugin system using wazero:

```go
// Plugin host implementation
type PluginHost struct {
    runtime  wazero.Runtime
    plugins  map[string]api.Module
    mu       sync.RWMutex
}

func NewPluginHost(ctx context.Context) *PluginHost {
    return &PluginHost{
        runtime: wazero.NewRuntime(ctx),
        plugins: make(map[string]api.Module),
    }
}

func (h *PluginHost) LoadPlugin(ctx context.Context, name string, wasmBytes []byte) error {
    h.mu.Lock()
    defer h.mu.Unlock()
    
    // Compile plugin
    compiled, err := h.runtime.CompileModule(ctx, wasmBytes)
    if err != nil {
        return fmt.Errorf("compile error: %w", err)
    }
    
    // Configure plugin environment
    config := wazero.NewModuleConfig().
        WithName(name).
        WithFS(os.DirFS(fmt.Sprintf("./plugins/%s", name)))
    
    // Instantiate plugin
    module, err := h.runtime.InstantiateModule(ctx, compiled, config)
    if err != nil {
        return fmt.Errorf("instantiation error: %w", err)
    }
    
    h.plugins[name] = module
    return nil
}

func (h *PluginHost) CallPlugin(ctx context.Context, name, function string, params ...uint64) ([]uint64, error) {
    h.mu.RLock()
    module, exists := h.plugins[name]
    h.mu.RUnlock()
    
    if !exists {
        return nil, fmt.Errorf("plugin %s not found", name)
    }
    
    fn := module.ExportedFunction(function)
    if fn == nil {
        return nil, fmt.Errorf("function %s not found", function)
    }
    
    return fn.Call(ctx, params...)
}
```

### 2. Sandboxed Code Execution

```go
// Secure execution environment for untrusted code
type Sandbox struct {
    runtime wazero.Runtime
    timeout time.Duration
}

func (s *Sandbox) Execute(wasmCode []byte, input []byte) ([]byte, error) {
    ctx, cancel := context.WithTimeout(context.Background(), s.timeout)
    defer cancel()
    
    // Compile with restrictions
    compiled, err := s.runtime.CompileModule(ctx, wasmCode)
    if err != nil {
        return nil, err
    }
    
    // Limited environment
    config := wazero.NewModuleConfig().
        WithFS(fstest.MapFS{}). // No file access
        WithStartFunctions().    // No auto-start
        WithName("sandbox")
    
    module, err := s.runtime.InstantiateModule(ctx, compiled, config)
    if err != nil {
        return nil, err
    }
    defer module.Close(ctx)
    
    // Write input to memory
    memory := module.Memory()
    memory.Write(0, input)
    
    // Execute sandboxed function
    results, err := module.ExportedFunction("process").Call(ctx, 0, uint64(len(input)))
    if err != nil {
        return nil, err
    }
    
    // Read output
    outputPtr := uint32(results[0])
    outputLen := uint32(results[1])
    output, _ := memory.Read(outputPtr, outputLen)
    
    return output, nil
}
```

### 3. Cross-Language Integration

```go
// Running Rust code in Go application
type RustIntegration struct {
    runtime wazero.Runtime
    module  api.Module
}

func NewRustIntegration(ctx context.Context, rustWasmPath string) (*RustIntegration, error) {
    wasmBytes, err := os.ReadFile(rustWasmPath)
    if err != nil {
        return nil, err
    }
    
    r := wazero.NewRuntime(ctx)
    
    // Register memory allocation functions for Rust
    _, err = r.NewHostModuleBuilder("env").
        NewFunctionBuilder().
        WithFunc(func(size uint32) uint32 {
            // Allocate memory for Rust
            return allocateMemory(size)
        }).
        Export("malloc").
        NewFunctionBuilder().
        WithFunc(func(ptr uint32) {
            // Free memory from Rust
            freeMemory(ptr)
        }).
        Export("free").
        Instantiate(ctx)
    
    if err != nil {
        return nil, err
    }
    
    module, err := r.Instantiate(ctx, wasmBytes)
    if err != nil {
        return nil, err
    }
    
    return &RustIntegration{
        runtime: r,
        module:  module,
    }, nil
}
```

## Comparison with Alternatives

### WebAssembly Runtimes for Go

| Feature | wazero | Wasmer-go | Wasmtime-go | Life | Wagon |
|---------|--------|-----------|-------------|------|-------|
| **CGO Required** | ❌ No | ✅ Yes | ✅ Yes | ❌ No | ❌ No |
| **Active Development** | ✅ Yes | ✅ Yes | ✅ Yes | ❌ No | ❌ No |
| **WASI Support** | ✅ Full | ✅ Full | ✅ Full | ⚠️ Partial | ❌ No |
| **Compiler Mode** | ✅ Yes | ✅ Yes | ✅ Yes | ❌ No | ❌ No |
| **Cross-Compilation** | ✅ Easy | ❌ Hard | ❌ Hard | ✅ Easy | ✅ Easy |
| **Binary Size** | Small | Large | Large | Small | Small |
| **Performance** | Good | Excellent | Excellent | Poor | Poor |
| **Go Integration** | Excellent | Good | Good | Good | Good |

### Decision Matrix

#### Choose wazero when:
- ✅ You need pure Go solution without CGO
- ✅ Cross-compilation is important
- ✅ Deployment simplicity matters
- ✅ Binary size needs to be minimal
- ✅ You want native Go features (context, channels)
- ✅ Building a plugin system
- ✅ Running in restricted environments (no CGO)

#### Choose Wasmer/Wasmtime when:
- ✅ Maximum performance is critical
- ✅ You're already using CGO
- ✅ Need cutting-edge WASM features
- ✅ Don't need cross-compilation
- ✅ Can manage complex dependencies

## CGO and Its Implications

### What is CGO?

CGO enables Go programs to call C code directly. It's a powerful feature but comes with significant trade-offs.

#### Basic CGO Example
```go
package main

/*
#include <math.h>
#include <stdio.h>

double calculate(double x) {
    return sqrt(x) * 2.0;
}
*/
import "C"
import "fmt"

func main() {
    result := C.calculate(C.double(16.0))
    fmt.Printf("Result: %f\n", float64(result))
}
```

### CGO Problems

1. **Build Complexity**
   - Requires C compiler (gcc, clang)
   - Platform-specific build flags
   - Dependency on system libraries

2. **Cross-Compilation Issues**
   ```bash
   # Without CGO (wazero)
   GOOS=linux GOARCH=arm64 go build  # Works!
   
   # With CGO (Wasmer-go, Wasmtime-go)
   GOOS=linux GOARCH=arm64 go build  # Fails without cross-compiler
   ```

3. **Deployment Challenges**
   - Need to ship shared libraries (.so, .dll, .dylib)
   - Library version management
   - Different libraries for different platforms

4. **Performance Overhead**
   - Go ↔ C boundary crossing cost
   - Stack switching overhead
   - Garbage collector coordination

5. **Debugging Difficulty**
   - Mixed Go/C stack traces
   - Complex error messages
   - Hard to profile

### Why wazero Avoids CGO

```
Traditional WASM Runtimes:
Go Code → CGO → C/Rust Runtime → Execute WASM
         ↑
    Complexity Point

wazero:
Go Code → Pure Go Runtime → Execute WASM
         ↑
    Simple & Direct
```

### Build Size Comparison

```bash
# Application using wazero (no CGO)
$ du -h myapp
2.1M    myapp

# Application using Wasmer-go (with CGO)
$ du -h myapp
45M     myapp
$ ldd myapp
    libwasmer.so => /usr/local/lib/libwasmer.so
    libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6
    libpthread.so.0 => /lib/x86_64-linux-gnu/libpthread.so.0
```

## Implementation Examples

### Example 1: Basic WebAssembly Execution

```go
package main

import (
    "context"
    "embed"
    "fmt"
    "log"
    
    "github.com/tetratelabs/wazero"
    "github.com/tetratelabs/wazero/imports/wasi_snapshot_preview1"
)

//go:embed module.wasm
var wasmModule []byte

func main() {
    ctx := context.Background()
    
    // Create runtime
    r := wazero.NewRuntime(ctx)
    defer r.Close(ctx)
    
    // Add WASI support
    wasi_snapshot_preview1.MustInstantiate(ctx, r)
    
    // Configure and instantiate module
    config := wazero.NewModuleConfig().
        WithStdout(os.Stdout).
        WithStderr(os.Stderr)
    
    module, err := r.InstantiateWithConfig(ctx, wasmModule, config)
    if err != nil {
        log.Fatal(err)
    }
    defer module.Close(ctx)
    
    // Call exported function
    add := module.ExportedFunction("add")
    results, err := add.Call(ctx, 5, 3)
    if err != nil {
        log.Fatal(err)
    }
    
    fmt.Printf("5 + 3 = %d\n", results[0])
}
```

### Example 2: Advanced Plugin System

```go
package main

import (
    "context"
    "fmt"
    "sync"
    
    "github.com/tetratelabs/wazero"
    "github.com/tetratelabs/wazero/api"
)

// Plugin interface all plugins must implement
type Plugin interface {
    Name() string
    Version() string
    Execute(ctx context.Context, input []byte) ([]byte, error)
}

// WASM plugin wrapper
type WasmPlugin struct {
    name    string
    version string
    module  api.Module
    execFn  api.Function
}

func (p *WasmPlugin) Name() string    { return p.name }
func (p *WasmPlugin) Version() string { return p.version }

func (p *WasmPlugin) Execute(ctx context.Context, input []byte) ([]byte, error) {
    // Allocate memory in WASM module
    allocFn := p.module.ExportedFunction("allocate")
    ptrResults, err := allocFn.Call(ctx, uint64(len(input)))
    if err != nil {
        return nil, err
    }
    ptr := uint32(ptrResults[0])
    
    // Write input to WASM memory
    memory := p.module.Memory()
    if !memory.Write(ptr, input) {
        return nil, fmt.Errorf("failed to write input")
    }
    
    // Execute plugin function
    results, err := p.execFn.Call(ctx, uint64(ptr), uint64(len(input)))
    if err != nil {
        return nil, err
    }
    
    // Read output
    outPtr := uint32(results[0])
    outLen := uint32(results[1])
    output, ok := memory.Read(outPtr, outLen)
    if !ok {
        return nil, fmt.Errorf("failed to read output")
    }
    
    // Free memory
    freeFn := p.module.ExportedFunction("deallocate")
    freeFn.Call(ctx, uint64(ptr))
    freeFn.Call(ctx, uint64(outPtr))
    
    return output, nil
}

// Plugin manager
type PluginManager struct {
    runtime wazero.Runtime
    plugins map[string]Plugin
    mu      sync.RWMutex
}

func NewPluginManager(ctx context.Context) *PluginManager {
    return &PluginManager{
        runtime: wazero.NewRuntime(ctx),
        plugins: make(map[string]Plugin),
    }
}

func (pm *PluginManager) LoadPlugin(ctx context.Context, name string, wasmBytes []byte) error {
    pm.mu.Lock()
    defer pm.mu.Unlock()
    
    // Compile module
    compiled, err := pm.runtime.CompileModule(ctx, wasmBytes)
    if err != nil {
        return fmt.Errorf("compilation failed: %w", err)
    }
    
    // Create host functions for plugin
    _, err = pm.runtime.NewHostModuleBuilder("host").
        NewFunctionBuilder().
        WithFunc(func(ctx context.Context, level uint32, msg string) {
            fmt.Printf("[%s] %s\n", name, msg)
        }).
        Export("log").
        Instantiate(ctx)
    
    if err != nil {
        return err
    }
    
    // Instantiate module
    module, err := pm.runtime.InstantiateModule(ctx, compiled,
        wazero.NewModuleConfig().WithName(name))
    if err != nil {
        return fmt.Errorf("instantiation failed: %w", err)
    }
    
    // Get metadata functions
    nameFn := module.ExportedFunction("plugin_name")
    versionFn := module.ExportedFunction("plugin_version")
    
    // Get plugin info
    nameResults, _ := nameFn.Call(ctx)
    versionResults, _ := versionFn.Call(ctx)
    
    plugin := &WasmPlugin{
        name:    readString(module.Memory(), uint32(nameResults[0])),
        version: readString(module.Memory(), uint32(versionResults[0])),
        module:  module,
        execFn:  module.ExportedFunction("execute"),
    }
    
    pm.plugins[name] = plugin
    return nil
}

func (pm *PluginManager) ExecutePlugin(ctx context.Context, name string, input []byte) ([]byte, error) {
    pm.mu.RLock()
    plugin, exists := pm.plugins[name]
    pm.mu.RUnlock()
    
    if !exists {
        return nil, fmt.Errorf("plugin %s not found", name)
    }
    
    return plugin.Execute(ctx, input)
}

func readString(memory api.Memory, ptr uint32) string {
    // Read null-terminated string from memory
    var result []byte
    for i := uint32(0); ; i++ {
        b, ok := memory.ReadByte(ptr + i)
        if !ok || b == 0 {
            break
        }
        result = append(result, b)
    }
    return string(result)
}
```

### Example 3: Performance-Optimized Configuration

```go
package main

import (
    "context"
    "runtime"
    "sync"
    
    "github.com/tetratelabs/wazero"
    "github.com/tetratelabs/wazero/api"
    "github.com/tetratelabs/wazero/sys"
)

type OptimizedRuntime struct {
    runtime      wazero.Runtime
    compiledPool sync.Pool
    modulePool   sync.Pool
}

func NewOptimizedRuntime(ctx context.Context) *OptimizedRuntime {
    // Configure for performance
    config := wazero.NewRuntimeConfig().
        WithCoreFeatures(api.CoreFeaturesV2).
        WithMemoryLimitPages(1024). // 64MB max per module
        WithCompilationCache(wazero.NewCompilationCache())
    
    r := wazero.NewRuntimeWithConfig(ctx, config)
    
    return &OptimizedRuntime{
        runtime: r,
        compiledPool: sync.Pool{
            New: func() interface{} {
                return make(map[string]wazero.CompiledModule)
            },
        },
        modulePool: sync.Pool{
            New: func() interface{} {
                return make([]api.Module, 0, runtime.NumCPU())
            },
        },
    }
}

func (or *OptimizedRuntime) ExecuteBatch(ctx context.Context, wasmBytes []byte, inputs [][]byte) ([][]byte, error) {
    // Compile once
    compiled, err := or.runtime.CompileModule(ctx, wasmBytes)
    if err != nil {
        return nil, err
    }
    
    // Process in parallel
    results := make([][]byte, len(inputs))
    errors := make([]error, len(inputs))
    
    var wg sync.WaitGroup
    semaphore := make(chan struct{}, runtime.NumCPU())
    
    for i := range inputs {
        wg.Add(1)
        semaphore <- struct{}{} // Limit concurrency
        
        go func(idx int) {
            defer wg.Done()
            defer func() { <-semaphore }()
            
            // Create isolated module instance
            module, err := or.runtime.InstantiateModule(ctx, compiled,
                wazero.NewModuleConfig().
                    WithName(fmt.Sprintf("worker_%d", idx)))
            if err != nil {
                errors[idx] = err
                return
            }
            defer module.Close(ctx)
            
            // Process input
            results[idx], errors[idx] = or.processWithModule(ctx, module, inputs[idx])
        }(i)
    }
    
    wg.Wait()
    
    // Check for errors
    for _, err := range errors {
        if err != nil {
            return nil, err
        }
    }
    
    return results, nil
}

func (or *OptimizedRuntime) processWithModule(ctx context.Context, module api.Module, input []byte) ([]byte, error) {
    memory := module.Memory()
    
    // Allocate memory
    allocFn := module.ExportedFunction("allocate")
    ptrResult, err := allocFn.Call(ctx, uint64(len(input)))
    if err != nil {
        return nil, err
    }
    ptr := uint32(ptrResult[0])
    
    // Write input
    if !memory.Write(ptr, input) {
        return nil, fmt.Errorf("memory write failed")
    }
    
    // Execute
    processFn := module.ExportedFunction("process")
    results, err := processFn.Call(ctx, uint64(ptr), uint64(len(input)))
    if err != nil {
        return nil, err
    }
    
    // Read output
    outPtr := uint32(results[0])
    outLen := uint32(results[1])
    output, ok := memory.Read(outPtr, outLen)
    if !ok {
        return nil, fmt.Errorf("memory read failed")
    }
    
    return output, nil
}
```

## Best Practices

### 1. Module Lifecycle Management

```go
// Always close modules and runtime
ctx := context.Background()
r := wazero.NewRuntime(ctx)
defer r.Close(ctx) // Important: prevents memory leaks

module, err := r.InstantiateModule(ctx, compiled, config)
if err != nil {
    return err
}
defer module.Close(ctx) // Important: frees module resources
```

### 2. Error Handling

```go
// Wrap errors with context
compiled, err := runtime.CompileModule(ctx, wasmBytes)
if err != nil {
    return fmt.Errorf("failed to compile WASM module: %w", err)
}

// Check memory operations
if !memory.Write(ptr, data) {
    return fmt.Errorf("memory write failed at offset %d", ptr)
}
```

### 3. Context Usage

```go
// Use context for timeouts
ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
defer cancel()

// Pass context through entire call chain
results, err := module.ExportedFunction("process").Call(ctx, params...)
if err != nil {
    if ctx.Err() == context.DeadlineExceeded {
        return nil, fmt.Errorf("WASM execution timeout")
    }
    return nil, err
}
```

### 4. Memory Management

```go
// Pre-allocate memory when possible
const bufferSize = 1024 * 1024 // 1MB
allocFn := module.ExportedFunction("allocate")
ptrResult, _ := allocFn.Call(ctx, bufferSize)
bufferPtr := uint32(ptrResult[0])

// Reuse buffer for multiple operations
for _, data := range dataSlices {
    memory.Write(bufferPtr, data)
    // Process data...
}

// Always free allocated memory
defer module.ExportedFunction("free").Call(ctx, uint64(bufferPtr))
```

### 5. Compilation Caching

```go
type ModuleCache struct {
    cache map[string]wazero.CompiledModule
    mu    sync.RWMutex
}

func (mc *ModuleCache) GetOrCompile(ctx context.Context, r wazero.Runtime, key string, wasmBytes []byte) (wazero.CompiledModule, error) {
    mc.mu.RLock()
    compiled, exists := mc.cache[key]
    mc.mu.RUnlock()
    
    if exists {
        return compiled, nil
    }
    
    mc.mu.Lock()
    defer mc.mu.Unlock()
    
    // Double-check after acquiring write lock
    if compiled, exists := mc.cache[key]; exists {
        return compiled, nil
    }
    
    compiled, err := r.CompileModule(ctx, wasmBytes)
    if err != nil {
        return nil, err
    }
    
    mc.cache[key] = compiled
    return compiled, nil
}
```

### 6. Security Considerations

```go
// Restrict module capabilities
config := wazero.NewModuleConfig().
    WithFS(fstest.MapFS{}).           // No file system access
    WithArgs().                        // No command line args
    WithEnv().                         // No environment variables
    WithStartFunctions().              // No auto-start functions
    WithMemoryLimitPages(16).          // 1MB memory limit
    WithSysNanosleep().                // Allow sleep
    WithSysWalltime().                 // Allow time access
    WithRandSource(rand.Reader).       // Secure random source
    WithName("sandboxed")

// Validate WASM before execution
if !isValidWASM(wasmBytes) {
    return fmt.Errorf("invalid WASM module")
}
```

## Future Roadmap

### Upcoming Features (Based on Community Discussions)

1. **WebAssembly Component Model**
   - Support for interface types
   - Better module composition
   - Standardized ABI

2. **Performance Improvements**
   - Continuous compiler optimizations
   - Better caching strategies
   - SIMD support expansion

3. **Enhanced WASI Support**
   - WASI Preview 2 implementation
   - Network capabilities
   - Threading support

4. **Developer Experience**
   - Better debugging tools
   - Performance profiling
   - Visual Studio Code extension

### Integration with Go Ecosystem

#### Go 1.21+ WASI Support
```go
// Native Go WASI compilation
GOOS=wasip1 GOARCH=wasm go build -o module.wasm

// Run with wazero
module, err := runtime.Instantiate(ctx, moduleBytes)
```

#### Go 1.24+ WebAssembly Export
```go
//go:wasmexport add
func add(a, b int32) int32 {
    return a + b
}

// Compile and use in wazero
```

## Conclusion

wazero represents a pragmatic approach to WebAssembly in Go, prioritizing operational simplicity and Go ecosystem integration over raw performance. Its zero-dependency architecture makes it ideal for:

1. **Plugin Systems**: Safe, portable extension mechanisms
2. **Cross-Platform Applications**: True write-once, run-anywhere
3. **Sandboxed Execution**: Secure code isolation
4. **Microservices**: Lightweight polyglot execution
5. **Edge Computing**: Small binary size, no dependencies

While synthetic benchmarks show wazero as slower than CGO-based alternatives, real-world applications often find the performance adequate, especially when considering:
- Elimination of CGO overhead
- Simplified deployment
- Reduced binary size
- Better debugging experience
- Native Go integration

For teams valuing operational excellence, deployment simplicity, and Go-native development experience, wazero offers compelling advantages that often outweigh its performance trade-offs.

## References

- [Official wazero Repository](https://github.com/tetratelabs/wazero)
- [wazero Documentation](https://wazero.io/)
- [WebAssembly Specification](https://webassembly.github.io/spec/)
- [WASI Specification](https://wasi.dev/)
- [Go WebAssembly Support](https://go.dev/wiki/WebAssembly)
- [Arcjet Production Case Study](https://blog.arcjet.com/lessons-from-running-webassembly-in-production-with-go-wazero/)
- [DoltHub WASM Integration](https://www.dolthub.com/blog/2023-05-19-wasm-in-go/)
- [go-plugin Framework](https://github.com/knqyf263/go-plugin)

---

*Document compiled: 2025-01-11*  
*Investigation conducted for: hail-mary project reference*