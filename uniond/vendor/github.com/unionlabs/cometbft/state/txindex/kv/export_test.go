package kv

func GetKeys(indexer *TxIndex) [][]byte {
	return getKeys(indexer)
}
