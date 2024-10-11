package types

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestNewStoreKVPairWriteListener(t *testing.T) {
	listener := NewMemoryListener()
	require.IsType(t, &MemoryListener{}, listener)
}

func TestOnWrite(t *testing.T) {
	listener := NewMemoryListener()

	testStoreKey := NewKVStoreKey("test_key")
	testKey := []byte("testing123")
	testValue := []byte("testing321")

	// test set
	listener.OnWrite(testStoreKey, testKey, testValue, false)
	outputKVPair := listener.PopStateCache()[0]
	expectedOutputKVPair := &StoreKVPair{
		Key:      testKey,
		Value:    testValue,
		StoreKey: testStoreKey.Name(),
		Delete:   false,
	}
	require.EqualValues(t, expectedOutputKVPair, outputKVPair)

	// test delete
	listener.OnWrite(testStoreKey, testKey, testValue, true)
	outputKVPair = listener.PopStateCache()[0]
	expectedOutputKVPair = &StoreKVPair{
		Key:      testKey,
		Value:    testValue,
		StoreKey: testStoreKey.Name(),
		Delete:   true,
	}
	require.EqualValues(t, expectedOutputKVPair, outputKVPair)
}
