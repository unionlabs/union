package db

import (
	"errors"
	"fmt"
	"regexp"
	"strconv"

	"github.com/google/orderedcode"
)

type LightStoreKeyLayout interface {
	// Implementations of ParseLBKey should create a copy of the key parameter,
	// rather than modify it in place.
	ParseLBKey(key []byte, storePrefix string) (height int64, err error)
	LBKey(height int64, prefix string) []byte
	SizeKey(prefix string) []byte
}

type v1LegacyLayout struct{}

// LBKey implements LightStoreKeyLayout.
func (v1LegacyLayout) LBKey(height int64, prefix string) []byte {
	return []byte(fmt.Sprintf("lb/%s/%020d", prefix, height))
}

// ParseLBKey implements LightStoreKeyLayout.
func (v1LegacyLayout) ParseLBKey(key []byte, _ string) (height int64, err error) {
	var part string
	part, _, height, err = parseKey(key)
	if part != "lb" {
		return 0, err
	}
	return height, nil
}

// SizeKey implements LightStoreKeyLayout.
func (v1LegacyLayout) SizeKey(_ string) []byte {
	return []byte("size")
}

var _ LightStoreKeyLayout = v1LegacyLayout{}

var keyPattern = regexp.MustCompile(`^(lb)/([^/]*)/([0-9]+)$`)

func parseKey(key []byte) (part string, prefix string, height int64, err error) {
	submatch := keyPattern.FindSubmatch(key)
	if submatch == nil {
		return "", "", 0, errors.New("not a light block key")
	}
	part = string(submatch[1])
	prefix = string(submatch[2])
	height, err = strconv.ParseInt(string(submatch[3]), 10, 64)
	if err != nil {
		return "", "", 0, err
	}
	return part, prefix, height, nil
}

const (
	// prefixes must be unique across all db's.
	prefixLightBlock = int64(11)
	prefixSize       = int64(12)
)

type v2Layout struct{}

// LBKey implements LightStoreKeyLayout.
func (v2Layout) LBKey(height int64, prefix string) []byte {
	key, err := orderedcode.Append(nil, prefix, prefixLightBlock, height)
	if err != nil {
		panic(err)
	}
	return key
}

// ParseLBKey implements LightStoreKeyLayout.
func (v2Layout) ParseLBKey(key []byte, storePrefix string) (height int64, err error) {
	var (
		dbPrefix         string
		lightBlockPrefix int64
	)
	remaining, err := orderedcode.Parse(string(key), &dbPrefix, &lightBlockPrefix, &height)
	if err != nil {
		err = fmt.Errorf("failed to parse light block key: %w", err)
	}
	if len(remaining) != 0 {
		err = fmt.Errorf("expected no remainder when parsing light block key but got: %s", remaining)
	}
	if lightBlockPrefix != prefixLightBlock {
		err = fmt.Errorf("expected light block prefix but got: %d", lightBlockPrefix)
	}
	if dbPrefix != storePrefix {
		err = fmt.Errorf("parsed key has a different prefix. Expected: %s, got: %s", storePrefix, dbPrefix)
	}
	return height, err
}

// SizeKey implements LightStoreKeyLayout.
func (v2Layout) SizeKey(prefix string) []byte {
	key, err := orderedcode.Append(nil, prefix, prefixSize)
	if err != nil {
		panic(err)
	}
	return key
}

var _ LightStoreKeyLayout = v2Layout{}
