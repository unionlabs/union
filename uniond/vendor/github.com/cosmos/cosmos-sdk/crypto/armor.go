package crypto

import (
	"bytes"
	"encoding/hex"
	"errors"
	"fmt"
	"io"

	"github.com/cometbft/cometbft/crypto"
	"golang.org/x/crypto/argon2"
	"golang.org/x/crypto/chacha20poly1305"
	"golang.org/x/crypto/openpgp/armor" //nolint:staticcheck //TODO: remove this dependency

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/codec/legacy"
	"github.com/cosmos/cosmos-sdk/crypto/keys/bcrypt"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/crypto/xsalsa20symmetric"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

const (
	blockTypePrivKey = "TENDERMINT PRIVATE KEY"
	blockTypeKeyInfo = "TENDERMINT KEY INFO"
	blockTypePubKey  = "TENDERMINT PUBLIC KEY"

	defaultAlgo = "secp256k1"

	headerVersion = "version"
	headerType    = "type"
)

var (
	kdfHeader = "kdf"
	kdfBcrypt = "bcrypt"
	kdfArgon2 = "argon2"
)

const (
	argon2Time    = 1
	argon2Memory  = 64 * 1024
	argon2Threads = 4
)

// BcryptSecurityParameter is security parameter var, and it can be changed within the lcd test.
// Making the bcrypt security parameter a var shouldn't be a security issue:
// One can't verify an invalid key by maliciously changing the bcrypt
// parameter during a runtime vulnerability. The main security
// threat this then exposes would be something that changes this during
// runtime before the user creates their key. This vulnerability must
// succeed to update this to that same value before every subsequent call
// to the keys command in future startups / or the attacker must get access
// to the filesystem. However, with a similar threat model (changing
// variables in runtime), one can cause the user to sign a different tx
// than what they see, which is a significantly cheaper attack then breaking
// a bcrypt hash. (Recall that the nonce still exists to break rainbow tables)
// For further notes on security parameter choice, see README.md
var BcryptSecurityParameter uint32 = 12

//-----------------------------------------------------------------
// add armor

// ArmorInfoBytes armor the InfoBytes
func ArmorInfoBytes(bz []byte) string {
	header := map[string]string{
		headerType:    "Info",
		headerVersion: "0.0.0",
	}

	return EncodeArmor(blockTypeKeyInfo, header, bz)
}

// ArmorPubKeyBytes armor the PubKeyBytes
func ArmorPubKeyBytes(bz []byte, algo string) string {
	header := map[string]string{
		headerVersion: "0.0.1",
	}
	if algo != "" {
		header[headerType] = algo
	}

	return EncodeArmor(blockTypePubKey, header, bz)
}

//-----------------------------------------------------------------
// remove armor

// UnarmorInfoBytes unarmor the InfoBytes
func UnarmorInfoBytes(armorStr string) ([]byte, error) {
	bz, header, err := unarmorBytes(armorStr, blockTypeKeyInfo)
	if err != nil {
		return nil, err
	}

	if header[headerVersion] != "0.0.0" {
		return nil, fmt.Errorf("unrecognized version: %v", header[headerVersion])
	}

	return bz, nil
}

// UnarmorPubKeyBytes returns the pubkey byte slice, a string of the algo type, and an error
func UnarmorPubKeyBytes(armorStr string) (bz []byte, algo string, err error) {
	bz, header, err := unarmorBytes(armorStr, blockTypePubKey)
	if err != nil {
		return nil, "", fmt.Errorf("couldn't unarmor bytes: %w", err)
	}

	switch header[headerVersion] {
	case "0.0.0":
		return bz, defaultAlgo, err
	case "0.0.1":
		if header[headerType] == "" {
			header[headerType] = defaultAlgo
		}

		return bz, header[headerType], err
	case "":
		return nil, "", errors.New("header's version field is empty")
	default:
		err = fmt.Errorf("unrecognized version: %v", header[headerVersion])
		return nil, "", err
	}
}

func unarmorBytes(armorStr, blockType string) (bz []byte, header map[string]string, err error) {
	bType, header, bz, err := DecodeArmor(armorStr)
	if err != nil {
		return
	}

	if bType != blockType {
		err = fmt.Errorf("unrecognized armor type %q, expected: %q", bType, blockType)
		return
	}

	return
}

//-----------------------------------------------------------------
// encrypt/decrypt with armor

// EncryptArmorPrivKey encrypt and armor the private key.
func EncryptArmorPrivKey(privKey cryptotypes.PrivKey, passphrase, algo string) string {
	saltBytes, encBytes := encryptPrivKey(privKey, passphrase)
	header := map[string]string{
		kdfHeader: kdfArgon2,
		"salt":    fmt.Sprintf("%X", saltBytes),
	}

	if algo != "" {
		header[headerType] = algo
	}

	armorStr := EncodeArmor(blockTypePrivKey, header, encBytes)

	return armorStr
}

func encryptPrivKey(privKey cryptotypes.PrivKey, passphrase string) (saltBytes, encBytes []byte) {
	saltBytes = crypto.CRandBytes(16)

	key := argon2.IDKey([]byte(passphrase), saltBytes, argon2Time, argon2Memory, argon2Threads, chacha20poly1305.KeySize)
	privKeyBytes := legacy.Cdc.MustMarshal(privKey)

	aead, err := chacha20poly1305.New(key)
	if err != nil {
		panic(errorsmod.Wrap(err, "error generating cypher from key"))
	}

	nonce := make([]byte, aead.NonceSize(), aead.NonceSize()+len(privKeyBytes)+aead.Overhead()) // Nonce is fixed to maintain consistency, each key is generated  at every encryption using a random salt.

	encBytes = aead.Seal(nil, nonce, privKeyBytes, nil)

	return saltBytes, encBytes
}

// UnarmorDecryptPrivKey returns the privkey byte slice, a string of the algo type, and an error
func UnarmorDecryptPrivKey(armorStr, passphrase string) (privKey cryptotypes.PrivKey, algo string, err error) {
	blockType, header, encBytes, err := DecodeArmor(armorStr)
	if err != nil {
		return privKey, "", err
	}

	if blockType != blockTypePrivKey {
		return privKey, "", fmt.Errorf("unrecognized armor type: %v", blockType)
	}

	if header[kdfHeader] != kdfBcrypt && header[kdfHeader] != kdfArgon2 {
		return privKey, "", fmt.Errorf("unrecognized KDF type: %v", header[kdfHeader])
	}

	if header["salt"] == "" {
		return privKey, "", errors.New("missing salt bytes")
	}

	saltBytes, err := hex.DecodeString(header["salt"])
	if err != nil {
		return privKey, "", fmt.Errorf("error decoding salt: %w", err)
	}

	privKey, err = decryptPrivKey(saltBytes, encBytes, passphrase, header[kdfHeader])

	if header[headerType] == "" {
		header[headerType] = defaultAlgo
	}

	return privKey, header[headerType], err
}

func decryptPrivKey(saltBytes, encBytes []byte, passphrase, kdf string) (privKey cryptotypes.PrivKey, err error) {
	// Key derivation
	var (
		key          []byte
		privKeyBytes []byte
	)

	// Since the argon2 key derivation and chacha encryption was implemented together, it is not possible to have mixed kdf and encryption algorithms
	switch kdf {
	case kdfArgon2:
		key = argon2.IDKey([]byte(passphrase), saltBytes, argon2Time, argon2Memory, argon2Threads, chacha20poly1305.KeySize)

		aead, err := chacha20poly1305.New(key)
		if err != nil {
			return privKey, errorsmod.Wrap(err, "Error generating aead cypher for key.")
		} else if len(encBytes) < aead.NonceSize() {
			return privKey, errorsmod.Wrap(nil, "Encrypted bytes length is smaller than aead nonce size.")
		}
		nonce := make([]byte, aead.NonceSize(), aead.NonceSize()+len(privKeyBytes)+aead.Overhead())
		privKeyBytes, err = aead.Open(nil, nonce, encBytes, nil) // Decrypt the message and check it wasn't tampered with.
		if err != nil {
			return privKey, sdkerrors.ErrWrongPassword
		}
	case kdfBcrypt:
		key, err = bcrypt.GenerateFromPassword(saltBytes, []byte(passphrase), BcryptSecurityParameter)
		if err != nil {
			return privKey, errorsmod.Wrap(err, "Error generating bcrypt cypher for key.")
		}
		key = crypto.Sha256(key) // Get 32 bytes
		privKeyBytes, err = xsalsa20symmetric.DecryptSymmetric(encBytes, key)

		if errors.Is(err, xsalsa20symmetric.ErrCiphertextDecrypt) {
			return privKey, sdkerrors.ErrWrongPassword
		}
	default:
		return privKey, errorsmod.Wrap(nil, fmt.Sprintf("Unrecognized key derivation function (kdf) header: %s.", kdf))
	}

	if err != nil {
		return privKey, err
	}

	return legacy.PrivKeyFromBytes(privKeyBytes)
}

//-----------------------------------------------------------------
// encode/decode with armor

func EncodeArmor(blockType string, headers map[string]string, data []byte) string {
	buf := new(bytes.Buffer)
	w, err := armor.Encode(buf, blockType, headers)
	if err != nil {
		panic(fmt.Errorf("could not encode ascii armor: %w", err))
	}
	_, err = w.Write(data)
	if err != nil {
		panic(fmt.Errorf("could not encode ascii armor: %w", err))
	}
	err = w.Close()
	if err != nil {
		panic(fmt.Errorf("could not encode ascii armor: %w", err))
	}
	return buf.String()
}

func DecodeArmor(armorStr string) (blockType string, headers map[string]string, data []byte, err error) {
	buf := bytes.NewBufferString(armorStr)
	block, err := armor.Decode(buf)
	if err != nil {
		return "", nil, nil, err
	}
	data, err = io.ReadAll(block.Body)
	if err != nil {
		return "", nil, nil, err
	}
	return block.Type, block.Header, data, nil
}
