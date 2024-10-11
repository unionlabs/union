package keyring_test

import (
	"log"

	"github.com/99designs/keyring"
)

func ExampleOpen() {
	// Use the best keyring implementation for your operating system
	kr, err := keyring.Open(keyring.Config{
		ServiceName: "my-service",
	})
	if err != nil {
		log.Fatal(err)
	}

	v, err := kr.Get("llamas")
	if err != nil {
		log.Fatal(err)
	}

	log.Printf("llamas was %v", v)
}
