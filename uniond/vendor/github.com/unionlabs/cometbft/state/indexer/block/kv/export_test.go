package kv

func GetKeys(indexer BlockerIndexer) [][]byte {
	return getKeys(indexer)
}
