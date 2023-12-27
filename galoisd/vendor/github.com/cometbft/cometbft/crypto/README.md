# crypto

crypto is the cryptographic package adapted for CometBFT's uses

## Importing it

To get the interfaces,
`import "github.com/cometbft/cometbft/crypto"`

For any specific algorithm, use its specific module e.g.
`import "github.com/cometbft/cometbft/crypto/ed25519"`

## Binary encoding

For Binary encoding, please refer to the [CometBFT encoding specification](https://github.com/cometbft/cometbft/blob/v0.38.x/spec/core/encoding.md).

## JSON Encoding

JSON encoding is done using CometBFT's internal json encoder. For more information on JSON encoding, please refer to [CometBFT JSON encoding](https://github.com/cometbft/cometbft/blob/v0.38.x/libs/json/doc.go)

```go
Example JSON encodings:

ed25519.PrivKey     - {"type":"tendermint/PrivKeyEd25519","value":"EVkqJO/jIXp3rkASXfh9YnyToYXRXhBr6g9cQVxPFnQBP/5povV4HTjvsy530kybxKHwEi85iU8YL0qQhSYVoQ=="}
ed25519.PubKey      - {"type":"tendermint/PubKeyEd25519","value":"AT/+aaL1eB0477Mud9JMm8Sh8BIvOYlPGC9KkIUmFaE="}
sr25519.PrivKeySr25519   - {"type":"tendermint/PrivKeySr25519","value":"xtYVH8UCIqfrY8FIFc0QEpAEBShSG4NT0zlEOVSZ2w4="}
sr25519.PubKeySr25519    - {"type":"tendermint/PubKeySr25519","value":"8sKBLKQ/OoXMcAJVxBqz1U7TyxRFQ5cmliuHy4MrF0s="}
crypto.PrivKeySecp256k1   - {"type":"tendermint/PrivKeySecp256k1","value":"zx4Pnh67N+g2V+5vZbQzEyRerX9c4ccNZOVzM9RvJ0Y="}
crypto.PubKeySecp256k1    - {"type":"tendermint/PubKeySecp256k1","value":"A8lPKJXcNl5VHt1FK8a244K9EJuS4WX1hFBnwisi0IJx"}
```
