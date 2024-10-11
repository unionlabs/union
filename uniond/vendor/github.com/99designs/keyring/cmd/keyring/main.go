package main

import (
	"flag"
	"fmt"
	"log"
	"os"

	"github.com/99designs/keyring"
)

func main() {
	serviceName := flag.String("service", "example", "The keyring service to use")
	keyName := flag.String("key", "example", "The key to use")
	backend := flag.String("backend", "", "A specific backend to use")
	debug := flag.Bool("debug", false, "Whether to enable debugging in keyring")
	listBackends := flag.Bool("list-backends", false, "Whether to list backends")

	// actions to take
	actionListKeys := flag.Bool("list-keys", false, "Whether to list keys")
	actionSetValue := flag.String("set", "", "The value to set")

	// keychain
	keychainName := flag.String("keychain", "login", "The keychain to search")

	flag.Parse()

	// Handle -list-backends
	if *listBackends {
		for _, b := range keyring.AvailableBackends() {
			fmt.Printf("%s\n", b)
		}
		os.Exit(0)
	}

	// Log to stderr
	log.SetOutput(os.Stderr)

	keyring.Debug = *debug

	var allowedBackends []keyring.BackendType
	if *backend != "" {
		if !hasBackend(*backend) {
			log.Fatalf("Backend %q isn't available. Use -list-backends to see what is.", *backend)
		}
		allowedBackends = append(allowedBackends, keyring.BackendType(*backend))
	} else {
		allowedBackends = keyring.AvailableBackends()
	}

	ring, err := keyring.Open(keyring.Config{
		ServiceName:     *serviceName,
		AllowedBackends: allowedBackends,
		KeychainName:    *keychainName,
	})
	if err != nil {
		log.Fatal(err)
	}

	switch {
	case *actionListKeys:
		if *debug {
			log.Printf("Listing keys in service %q in backend %q",
				*serviceName, allowedBackends[0])
		}
		keys, err := ring.Keys()
		if err != nil {
			log.Fatalf("Failed to list keys: %#v", err)
		}
		for _, key := range keys {
			fmt.Printf("%s\n", key)
		}

	case *actionSetValue != "":
		if *debug {
			log.Printf("Setting key %q in service %q in backend %q",
				*keyName, *serviceName, allowedBackends[0])
		}
		err := ring.Set(keyring.Item{
			Key:  *keyName,
			Data: []byte(*actionSetValue),
		})
		if err != nil {
			log.Fatal(err)
		}

	default:
		if *debug {
			log.Printf("Getting key %q in service %q in backend %q",
				*keyName, *serviceName, allowedBackends[0])
		}

		i, err := ring.Get(*keyName)
		if err != nil {
			log.Fatal(err)
		}
		fmt.Printf("%s", i.Data)
	}
}

func hasBackend(key string) bool {
	for _, b := range keyring.AvailableBackends() {
		if keyring.BackendType(key) == b {
			return true
		}
	}

	return false
}
