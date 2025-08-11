package main

import (
	"context"
	"fmt"
	"log"

	"github.com/ashigirl96/hail-mary/internal/wasm"
)

func main() {
	ctx := context.Background()

	module, err := wasm.NewWasmModule(ctx)
	if err != nil {
		log.Fatal(err)
	}
	defer module.Close(ctx)

	message, err := module.HelloWorld(ctx)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Printf("WASM returned: %s\n", message)
}
