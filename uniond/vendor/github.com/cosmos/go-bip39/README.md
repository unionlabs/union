# go-bip39

A golang implementation of the BIP0039 spec for mnemonic seeds

## Fork

This is a fork of github.com/tyler-smith/go-bip39 from right after the fixes
from bartekn for `MnemonicToByteArray` were merged
(commit hash: 52158e4697b87de16ed390e1bdaf813e581008fa).

The `tyler-smith` repo is undergoing significant refactoring at present that we may
not want (eg. some vars becoming private).

## Credits

English wordlist and test vectors are from the standard Python BIP0039 implementation
from the Trezor guys: [https://github.com/trezor/python-mnemonic](https://github.com/trezor/python-mnemonic)

## Example

```go
package main

import (
  "github.com/tyler-smith/go-bip39"
  "github.com/tyler-smith/go-bip32"
  "fmt"
)

func main(){
  // Generate a mnemonic for memorization or user-friendly seeds
  entropy, _ := bip39.NewEntropy(256)
  mnemonic, _ := bip39.NewMnemonic(entropy)

  // Generate a Bip32 HD wallet for the mnemonic and a user supplied password
  seed := bip39.NewSeed(mnemonic, "Secret Passphrase")

  masterKey, _ := bip32.NewMasterKey(seed)
  publicKey := masterKey.PublicKey()

  // Display mnemonic and keys
  fmt.Println("Mnemonic: ", mnemonic)
  fmt.Println("Master private key: ", masterKey)
  fmt.Println("Master public key: ", publicKey)
}
```
