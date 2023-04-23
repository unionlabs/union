package schnorrkel

import (
	"crypto/sha512"
	"errors"
	"math/big"
	"strings"

	bip39 "github.com/cosmos/go-bip39"
	"golang.org/x/crypto/pbkdf2"
)

// WARNING:  Non-standard BIP39 Implementation
// Designed for compatibility with the Rust substrate-bip39 library

// MiniSecretFromMnemonic returns a go-schnorrkel MiniSecretKey from a bip39 mnemonic
func MiniSecretFromMnemonic(mnemonic string, password string) (*MiniSecretKey, error) {
	seed, err := SeedFromMnemonic(mnemonic, password)
	if err != nil {
		return nil, err
	}
	var secret [32]byte
	copy(secret[:], seed[:32])
	return NewMiniSecretKeyFromRaw(secret)
}

// SeedFromMnemonic returns a 64-byte seed from a bip39 mnemonic
func SeedFromMnemonic(mnemonic string, password string) ([64]byte, error) {
	entropy, err := MnemonicToEntropy(mnemonic)
	if err != nil {
		return [64]byte{}, err
	}

	if len(entropy) < 16 || len(entropy) > 32 || len(entropy)%4 != 0 {
		return [64]byte{}, errors.New("invalid entropy")
	}

	bz := pbkdf2.Key(entropy, []byte("mnemonic"+password), 2048, 64, sha512.New)
	var bzArr [64]byte
	copy(bzArr[:], bz[:64])

	return bzArr, nil
}

// MnemonicToEntropy takes a mnemonic string and reverses it to the entropy
// An error is returned if the mnemonic is invalid.
func MnemonicToEntropy(mnemonic string) ([]byte, error) {
	_, err := bip39.MnemonicToByteArray(mnemonic)
	if err != nil {
		return nil, err
	}

	mnemonicSlice := strings.Split(mnemonic, " ")
	bitSize := len(mnemonicSlice) * 11
	checksumSize := bitSize % 32
	b := big.NewInt(0)
	modulo := big.NewInt(2048)
	for _, v := range mnemonicSlice {
		index, _ := bip39.ReverseWordMap[v]
		add := big.NewInt(int64(index))
		b = b.Mul(b, modulo)
		b = b.Add(b, add)
	}
	checksumModulo := big.NewInt(0).Exp(big.NewInt(2), big.NewInt(int64(checksumSize)), nil)
	entropy, _ := big.NewInt(0).DivMod(b, checksumModulo, big.NewInt(0))

	entropyHex := entropy.Bytes()

	// Add padding (no extra byte, entropy itself does not contain checksum)
	entropyByteSize := (bitSize - checksumSize) / 8
	if len(entropyHex) != entropyByteSize {
		tmp := make([]byte, entropyByteSize)
		diff := entropyByteSize - len(entropyHex)
		for i := 0; i < len(entropyHex); i++ {
			tmp[i+diff] = entropyHex[i]
		}
		entropyHex = tmp
	}

	return entropyHex, nil
}
