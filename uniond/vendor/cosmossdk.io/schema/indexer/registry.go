package indexer

import "fmt"

// Register registers an indexer type with the given initialization function.
func Register(indexerType string, descriptor Initializer) {
	if _, ok := indexerRegistry[indexerType]; ok {
		panic(fmt.Sprintf("indexer %s already registered", indexerType))
	}

	if descriptor.InitFunc == nil {
		panic(fmt.Sprintf("indexer %s has no initialization function", indexerType))
	}

	indexerRegistry[indexerType] = descriptor
}

var indexerRegistry = map[string]Initializer{}
