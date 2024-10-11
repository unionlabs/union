package address

import (
	"errors"
	"fmt"
	"strings"
	"sync"

	"github.com/hashicorp/golang-lru/simplelru"

	"cosmossdk.io/core/address"
	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/internal/conv"
	sdkAddress "github.com/cosmos/cosmos-sdk/types/address"
	"github.com/cosmos/cosmos-sdk/types/bech32"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

var errEmptyAddress = errors.New("empty address string is not allowed")

var (
	_ address.Codec = &Bech32Codec{}
	_ address.Codec = &cachedBech32Codec{}
)

type Bech32Codec struct {
	Bech32Prefix string
}

type cachedBech32Codec struct {
	codec Bech32Codec
	mu    *sync.Mutex
	cache *simplelru.LRU
}

type CachedCodecOptions struct {
	Mu  *sync.Mutex
	Lru *simplelru.LRU
}

func NewBech32Codec(prefix string) address.Codec {
	return &Bech32Codec{Bech32Prefix: prefix}
}

func NewCachedBech32Codec(prefix string, opts CachedCodecOptions) (address.Codec, error) {
	var err error
	ac := Bech32Codec{prefix}
	if opts.Mu == nil && opts.Lru == nil {
		opts.Mu = new(sync.Mutex)
		opts.Lru, err = simplelru.NewLRU(256, nil)
		if err != nil {
			return nil, fmt.Errorf("failed to create LRU cache: %w", err)
		}
	} else if opts.Mu == nil && opts.Lru != nil {
		// The LRU cache uses a map internally. Without a mutex, concurrent access to this map can lead to race conditions.
		// Therefore, a mutex is required to ensure thread-safe operations on the LRU cache.
		return nil, errors.New("mutex must be provided alongside the LRU cache")
	} else if opts.Mu != nil && opts.Lru == nil {
		opts.Lru, err = simplelru.NewLRU(256, nil)
		if err != nil {
			return nil, fmt.Errorf("failed to create LRU cache: %w", err)
		}
	}

	return cachedBech32Codec{
		codec: ac,
		cache: opts.Lru,
		mu:    opts.Mu,
	}, nil
}

// StringToBytes encodes text to bytes
func (bc Bech32Codec) StringToBytes(text string) ([]byte, error) {
	if len(strings.TrimSpace(text)) == 0 {
		return []byte{}, errEmptyAddress
	}

	hrp, bz, err := bech32.DecodeAndConvert(text)
	if err != nil {
		return nil, err
	}

	if len(bz) > sdkAddress.MaxAddrLen {
		return nil, errorsmod.Wrapf(sdkerrors.ErrUnknownAddress, "address max length is %d, got %d", sdkAddress.MaxAddrLen, len(bz))
	}

	if hrp != bc.Bech32Prefix {
		return nil, errorsmod.Wrapf(sdkerrors.ErrLogic, "hrp does not match bech32 prefix: expected '%s' got '%s'", bc.Bech32Prefix, hrp)
	}

	return bz, nil
}

// BytesToString decodes bytes to text
func (bc Bech32Codec) BytesToString(bz []byte) (string, error) {
	if len(bz) == 0 {
		return "", nil
	}

	text, err := bech32.ConvertAndEncode(bc.Bech32Prefix, bz)
	if err != nil {
		return "", err
	}

	if len(bz) > sdkAddress.MaxAddrLen {
		return "", errorsmod.Wrapf(sdkerrors.ErrUnknownAddress, "address max length is %d, got %d", sdkAddress.MaxAddrLen, len(bz))
	}

	return text, nil
}

func (cbc cachedBech32Codec) BytesToString(bz []byte) (string, error) {
	if len(bz) == 0 {
		return "", nil
	}

	key := conv.UnsafeBytesToStr(bz)
	cbc.mu.Lock()
	defer cbc.mu.Unlock()

	addrs, ok := cbc.cache.Get(key)
	if !ok {
		addrs = make(map[string]string)
		cbc.cache.Add(key, addrs)
	}

	addrMap, ok := addrs.(map[string]string)
	if !ok {
		return "", fmt.Errorf("cache contains non-map[string]string value for key %s", key)
	}

	addr, ok := addrMap[cbc.codec.Bech32Prefix]
	if !ok {
		var err error
		addr, err = cbc.codec.BytesToString(bz)
		if err != nil {
			return "", err
		}
		addrMap[cbc.codec.Bech32Prefix] = addr
	}

	return addr, nil
}

func (cbc cachedBech32Codec) StringToBytes(text string) ([]byte, error) {
	cbc.mu.Lock()
	defer cbc.mu.Unlock()

	if addr, ok := cbc.cache.Get(text); ok {
		return addr.([]byte), nil
	}

	addr, err := cbc.codec.StringToBytes(text)
	if err != nil {
		return nil, err
	}
	cbc.cache.Add(text, addr)

	return addr, nil
}
