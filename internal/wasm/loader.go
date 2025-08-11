package wasm

import (
	"context"
	_ "embed"
	"fmt"

	"github.com/tetratelabs/wazero"
	"github.com/tetratelabs/wazero/api"
)

//go:embed module.wasm
var wasmModule []byte

type WasmModule struct {
	runtime wazero.Runtime
	module  api.Module
}

func NewWasmModule(ctx context.Context) (*WasmModule, error) {
	r := wazero.NewRuntime(ctx)

	module, err := r.Instantiate(ctx, wasmModule)
	if err != nil {
		r.Close(ctx)
		return nil, fmt.Errorf("failed to instantiate module: %w", err)
	}

	return &WasmModule{
		runtime: r,
		module:  module,
	}, nil
}

func (w *WasmModule) HelloWorld(ctx context.Context) (string, error) {
	helloFn := w.module.ExportedFunction("hello_world")
	freeFn := w.module.ExportedFunction("free_string")

	if helloFn == nil {
		return "", fmt.Errorf("hello_world function not found")
	}
	if freeFn == nil {
		return "", fmt.Errorf("free_string function not found")
	}

	results, err := helloFn.Call(ctx)
	if err != nil {
		return "", err
	}

	if len(results) == 0 {
		return "", fmt.Errorf("no results returned from hello_world")
	}

	ptr := results[0]
	if ptr == 0 {
		return "", fmt.Errorf("null pointer returned from hello_world")
	}

	// Read string from memory
	message := readString(w.module.Memory(), uint32(ptr))

	// Free the string in WASM memory
	_, _ = freeFn.Call(ctx, ptr)

	return message, nil
}

func readString(memory api.Memory, ptr uint32) string {
	buf, ok := memory.Read(ptr, memory.Size()-ptr)
	if !ok {
		return ""
	}

	for i, b := range buf {
		if b == 0 {
			return string(buf[:i])
		}
	}
	return string(buf)
}

func (w *WasmModule) Close(ctx context.Context) error {
	return w.runtime.Close(ctx)
}
