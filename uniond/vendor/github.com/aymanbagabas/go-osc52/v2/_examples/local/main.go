package main

import (
	"fmt"

	"github.com/aymanbagabas/go-osc52"
)

func main() {
	str := "hello world"
	osc52.Copy(str)
	fmt.Printf("Copied %q!", str)
}
