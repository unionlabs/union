# ADR 013: Need for symmetric cryptography

## Context

We require symmetric ciphers to handle how we encrypt keys in the sdk,
and to potentially encrypt `priv_validator.json` in tendermint.

Currently we use AEAD's to support symmetric encryption,
which is great since we want data integrity in addition to privacy and authenticity.
We don't currently have a scenario where we want to encrypt without data integrity,
so it is fine to optimize our code to just use AEAD's.
Currently there is not a way to switch out AEAD's easily, this ADR outlines a way
to easily swap these out.

### How do we encrypt with AEAD's

AEAD's typically require a nonce in addition to the key.
For the purposes we require symmetric cryptography for,
we need encryption to be stateless.
Because of this we use random nonces.
(Thus the AEAD must support random nonces)

We currently construct a random nonce, and encrypt the data with it.
The returned value is `nonce || encrypted data`.
The limitation of this is that does not provide a way to identify
which algorithm was used in encryption.
Consequently decryption with multiple algorithms is sub-optimal.
(You have to try them all)

## Decision

We should create the following two methods in a new `crypto/encoding/symmetric` package:

```golang
func Encrypt(aead cipher.AEAD, plaintext []byte) (ciphertext []byte, err error)
func Decrypt(key []byte, ciphertext []byte) (plaintext []byte, err error)
func Register(aead cipher.AEAD, algo_name string, NewAead func(key []byte) (cipher.Aead, error)) error
```

This allows you to specify the algorithm in encryption, but not have to specify
it in decryption.
This is intended for ease of use in downstream applications, in addition to people
looking at the file directly.
One downside is that for the encrypt function you must have already initialized an AEAD,
but I don't really see this as an issue.

If there is no error in encryption, Encrypt will return `algo_name || nonce || aead_ciphertext`.
`algo_name` should be length prefixed, using standard varuint encoding.
This will be binary data, but that's not a problem considering the nonce and ciphertext are also binary.

This solution requires a mapping from aead type to name.
We can achieve this via reflection.

```golang
func getType(myvar interface{}) string {
    if t := reflect.TypeOf(myvar); t.Kind() == reflect.Ptr {
        return "*" + t.Elem().Name()
    } else {
        return t.Name()
    }
}
```

Then we maintain a map from the name returned from `getType(aead)` to `algo_name`.

In decryption, we read the `algo_name`, and then instantiate a new AEAD with the key.
Then we call the AEAD's decrypt method on the provided nonce/ciphertext.

`Register` allows a downstream user to add their own desired AEAD to the symmetric package.
It will error if the AEAD name is already registered.
This prevents a malicious import from modifying / nullifying an AEAD at runtime.

## Implementation strategy

The golang implementation of what is proposed is rather straight forward.
The concern is that we will break existing private keys if we just switch to this.
If this is concerning, we can make a simple script which doesn't require decoding privkeys,
for converting from the old format to the new one.

## Status

Proposed.

## Consequences

### Positive

- Allows us to support new AEAD's, in a way that makes decryption easier
- Allows downstream users to add their own AEAD

### Negative

- We will have to break all private keys stored on disk.
  They can be recovered using seed words, and upgrade scripts are simple.

### Neutral

- Caller has to instantiate the AEAD with the private key.
  However it forces them to be aware of what signing algorithm they are using, which is a positive.
