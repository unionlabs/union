# ADR 001: Crypto Providers

## Change log

* July 1st 2024: Implementation proposal (Zondax AG: @raynaudoe @juliantoledano @jleni @educlerici-zondax @lucaslopezf)

## Status

PROPOSED

## Abstract

This ADR proposes the refactoring of the existing cryptographic code to support multiple cryptographic curves for signing and verification processes. With this update, we aim to facilitate the integration of new cryptographic curves through clean and simple interfaces. Additionally, support for Hardware Security Modules (HSM) is introduced as a complementary enhancement in this redesign.

## Introduction

The introduction of multi-curve support for the Interchain Stack offers significant advantages. Developers can choose the most appropriate curve based on security, performance, and compatibility requirements. This flexibility enhances the application's ability to adapt to evolving security standards and optimizes performance for specific use cases, helping to future-proofing the Interchain apps cryptographic capabilities.

Special focus has been placed on the following key aspects:

* modularity
* extensibility
* security
* maintainability
* developer experience

On this document we'll introduce the concept of the `CryptoProvider` interface, which acts as a centralized controller for cryptographic operations, encapsulating the APIs for signing, verifying, and hashing functionalities. It abstracts the underlying cryptographic implementations, enabling a modular and extensible architecture. This allows users to easily switch between different cryptographic implementations without impacting the rest of the system.

Key capabilities include:

* **Signing**: Generate digital signatures for messages using various cryptographic curves.
* **Verifying**: Validate digital signatures against messages and public keys.
* **Hashing**: Perform hashing operations with different algorithms.
* **Hardware Security Module (HSM) Support**: Integrate with hardware devices and cloud-based HSMs for enhanced security.
* **Remote Signers**: Support for remote cryptographic operations, enabling interaction with secure, remote environments or cloud-based services.

### Glossary

1. **Interface**: In the context of this document, "interface" refers to Go's interface.

2. **Module**: In this document, "module" refers to a Go module.

3. **Package**: In the context of Go, a "package" refers to a unit of code organization.

## Objectives

The key objectives for this proposal are:

Modular Design Philosophy

* Establish a flexible and extensible foundation using interfaces to enable the seamless integration of various cryptographic curves.

* Restructure, Refactor, and Decouple: Update the codebase to ensure modularity and future adaptability.

Documentation & Community Engagement

* Enhance documentation to ensure clarity, establish a good practices protocol and promote community engagement, providing a platform for feedback and collaborative growth.

Backward Compatibility & Migration

* Prioritize compatibility with previous version to avoid disruptions for existing users.

* Design and propose a suitable migration path, ensuring transitions are as seamless as possible.

Developer-Centric Approach

* Prioritize clear, intuitive interfaces and best-practice design principles.

Quality Assurance

* Enhanced Test Coverage: Improve testing methodologies to ensure the robustness and reliability of the module.

* Conduct an Audit: After implementation, perform a comprehensive audit to identify potential vulnerabilities and ensure the module's security and stability.

## Technical Goals

Multi-curve support:

* Support for a wide range of cryptographic curves to be integrated seamlessly into any Interchain app in a modular way.

Wide Hardware Device & Cloud-based HSM Interface Support:

* Design a foundational interface for various hardware devices (Ledger, YubiKey, Thales, etc.) and cloud-based HSMs (Amazon, Azure) to support both current and future implementations.

Testing:

* Design an environment for testing, ensuring developers can validate integrations without compromising system integrity.

## Proposed architecture

In this section, we will first introduce the concept of a `CryptoProvider`, which serves as the main API. Following this, we will present the detailed components that make up the `CryptoProvider`. Lastly, we will introduce the storage and persistence layer, providing code snippets for each component to illustrate their implementation.

### Crypto Provider

This **interface** acts as a centralized controller, encapsulating the APIs for the **signing**, **verifying** and **hashing** functionalities. It acts as the main API with which the apps will interact with

By abstracting the underlying cryptographic functionalities, `CryptoProvider` enables a modular and extensible architecture, aka 'pluggable cryptography'. It allows users to easily switch between different cryptographic implementations without impacting the rest of the system.

The `CryptoProvider` interface includes getters for essential cryptographic functionalities and its metadata:

```go
// CryptoProvider aggregates the functionalities of signing, verifying, and hashing, and provides metadata.
type CryptoProvider interface {
    // GetSigner returns an instance of Signer.
    GetSigner() Signer

    // GetVerifier returns an instance of Verifier.
    GetVerifier() Verifier

    // GetHasher returns an instance of Hasher.
    GetHasher() Hasher

    // Metadata returns metadata for the crypto provider.
    Metadata() ProviderMetadata
}
```

#### Components

The components defined here are designed to act as *wrappers* around the underlying proper functions. This architecture ensures that the actual cryptographic operations such as signing, hashing, and verifying are delegated to the specialized functions, that are implementation dependant. These wrapper components facilitate a clean and modular approach by abstracting the complexity of direct cryptographic function calls.

In all of the interface's methods, we add an *options* input parameter of type `map[string]any`, designed to provide a flexible and dynamic way to pass various options and configurations to the `Sign`, `Verify`, and `Hash` functions. This approach allows developers to customize these processes by including any necessary parameters that might be required by specific algorithms or operational contexts. However, this requires that a type assertion for each option be performed inside the function's implementation.

##### Signer

Interface responsible for signing a message and returning the generated signature.
The `SignerOpts` map allows for flexible and dynamic configuration of the signing process.
This can include algorithm-specific parameters or any other contextual information
that might be necessary for the signing operation.

```go
// Signer represents a general interface for signing messages.
type Signer interface {
    // Sign takes a signDoc as input and returns the digital signature.
    Sign(signDoc []byte, options SignerOpts) (Signature, error)
}

type SignerOpts = map[string]any
```

###### Signature

```go
// Signature represents a general interface for a digital signature.
type Signature interface {
    // Bytes returns the byte representation of the signature.
    Bytes() []byte

    // Equals checks if two signatures are identical.
    Equals(other Signature) bool
}
```

##### Verifier

Verifies if given a message belongs to a public key by validating against its respective signature.

```go
// Verifier represents a general interface for verifying signatures.
type Verifier interface {
    // Verify checks the digital signature against the message and a public key to determine its validity.
    Verify(signature Signature, signDoc []byte, pubKey PubKey, options VerifierOpts) (bool, error)
}

type VerifierOpts = map[string]any
```

##### Hasher

This interface allows to have a specific hashing algorithm.

```go
// Hasher represents a general interface for hashing data.
type Hasher interface {
    // Hash takes an input byte array and returns the hashed output as a byte array.
    Hash(input []byte, options HasherOpts) (output []byte, err error)
}

type HasherOpts = map[string]any
```

##### Metadata

The metadata allows uniquely identifying a `CryptoProvider` and also stores its configurations.

```go
// ProviderMetadata holds metadata about the crypto provider.
type ProviderMetadata struct {
    Name    string
    Type    string
    Version *semver.Version // Using semver type for versioning
    Config  map[string]any
}
```

##### Public Key

*Note:* Here we decoupled the `Address` type from its corresponding `PubKey`. The corresponding codec step is proposed to be abstracted out from the `CryptoProvider` layer.

```go
type PubKey interface {
   Bytes() []byte
   Equals(other PubKey) bool
   Type() string
}
```

##### Private Key

*Note*: For example, in hardware wallets, the `PrivKey` interface acts only as a *reference* to the real data. This is a design consideration and may be subject to change during implementation.

Future enhancements could include additional security functions such as **zeroing** memory after private key usage to further enhance security measures.

```go
type PrivKey interface {
   Bytes() []byte
   PubKey() PubKey
   Equals(other PrivKey) bool
   Type() string
}
```

##### Storage and persistence

The storage and persistence layer is tasked with storing a `CryptoProvider`s. Specifically, this layer must:

* Securely store the crypto provider's associated private key (only if stored locally, otherwise a reference to the private key will be stored instead).
* Store the `ProviderMetadata` struct which contains the data that distinguishes that provider.

The purpose of this layer is to ensure that upon retrieval of the persisted data, we can access the provider's type, version, and specific configuration (which varies based on the provider type). This information will subsequently be utilized to initialize the appropriate factory.

Proposed alternatives:

* Using JSON file with a specification to store the crypto providers in a JSON format.
* Using Protobuf to store this information into a convenient Protobuf message. Apps that already use Protobuf might find this approach easier to integrate.


Below is the proposed protobuf message definition:

###### Protobuf message structure

```protobuf

// cryptoprovider.proto

syntax = "proto3";

package crypto;

import "google/protobuf/any.proto";

// CryptoProvider holds all necessary information to instantiate and configure a CryptoProvider.
message CryptoProvider {
    string name = 1; // (unique) name of the crypto provider.
    google.protobuf.Any pub_key = 2;
    string type = 3;       // Type of the crypto provider
    string version = 4;    // Version (semver format)
    map<string, bytes> config = 5;  // Configuration data with byte array values
    google.protobuf.Any privKey = 6; // Optional if key is stored locally
}
```

<b>name</b>:
Specifies the unique name of the crypto provider. This name is used to identify and reference the specific crypto provider instance.

<b>pub_key (google.protobuf.Any)</b>:
Holds the public key associated with the crypto provider.

<b>type</b>:
Specifies the type of the crypto provider. This field is used to identify and differentiate between various crypto provider implementations. Examples: `ledger`, `AWSCloudHSM`, `local-secp256k1`

<b>version</b>:
Indicates the version of the crypto provider using semantic versioning.

<b>configuration (map<string, bytes>)</b>:
Contains serialized configuration data as key-value pairs, where the key is a string and the value is a byte array.

<b>privKey (google.protobuf.Any)</b>:
An optional field that can store a private key if it is managed locally.

##### Creating and loading a `CryptoProvider`

For creating providers, we propose a *factory pattern* and a *registry* for these builders.

Below, we present the proposed interfaces and code snippets to illustrate the proposed architecture.

```go
// CryptoProviderFactory is a factory interface for creating CryptoProviders.
// Must be implemented by each CryptoProvider.
type CryptoProviderFactory interface {
    CreateFromJSON(data []byte) (CryptoProvider, error)
    CreateFromProto(data []byte) (CryptoProvider, error)
    CreateFromConfig(metadata ProviderMetadata) (CryptoProvider, error)
    Type() string
}
```

##### Illustrative Code Snippets

 The following code snippet demonstrates a provider **factory** and builder **registry**. Please note that this is for illustration purposes only. During actual implementation, these interfaces and methods may be subject to change and optimization.

```go
// crypto/v2/providerFactory.go

type Factory struct {
    providerFactories map[string]CryptoProviderFactory
}

// NewFactory creates a new Factory instance and initializes the providerFactories map.
func NewFactory() *Factory {
    return &Factory{
        providerFactories: make(map[string]CryptoProviderFactory),
    }
}


// RegisterCryptoProviderFactory is a function that registers a CryptoProviderFactory for its corresponding type.
func (f *Factory) RegisterCryptoProviderFactory(factory CryptoProviderFactory) string {    
    providerType := factory.Type()
    f.providerFactories[providerType] = factory
    return providerType
}


// CreateCryptoProviderFromJSON creates a CryptoProvider based on the provided JSON data.
func (f *Factory) CreateCryptoProviderFromJSON(data []byte) (CryptoProvider, error) {
    if data == nil {
        return nil, fmt.Errorf("data cannot be nil")
    }

    // Assuming the JSON data contains a type field to determine the provider type
    var metadata ProviderMetadata
    if err := json.Unmarshal(data, &metadata); err != nil {
        return nil, fmt.Errorf("failed to unmarshal JSON data: %v", err)
    }

    factory, exists := f.providerFactories[metadata.Type]
    if !exists {
        return nil, fmt.Errorf("no factory registered for provider type %s", metadata.Type)
    }

    return factory.CreateFromJSON(data)
}

// CreateCryptoProviderFromProto creates a CryptoProvider based on the provided Proto data.
func (f *Factory) CreateCryptoProviderFromProto(data []byte) (CryptoProvider, error) {
    if data == nil {
        return nil, fmt.Errorf("data cannot be nil")
    }

    // Assuming the Proto data contains a type field to determine the provider type
    var metadata ProviderMetadata
    if err := proto.Unmarshal(data, &metadata); err != nil {
        return nil, fmt.Errorf("failed to unmarshal Proto data: %v", err)
    }

    factory, exists := f.providerFactories[metadata.Type]
    if !exists {
        return nil, fmt.Errorf("no factory registered for provider type %s", metadata.Type)
    }

    return factory.CreateFromProto(data)
}

// CreateCryptoProviderFromConfig creates a CryptoProvider based on the provided ProviderMetadata.
func (f *Factory) CreateCryptoProviderFromConfig(config ProviderMetadata) (CryptoProvider, error) {
    if config.Type == "" {
        return nil, fmt.Errorf("config type cannot be empty")
    }

    factory, exists := f.providerFactories[config.Type]
    if !exists {
        return nil, fmt.Errorf("no factory registered for provider type %s", config.Type)
    }

    return factory.CreateFromConfig(config)
}
```

**Example**: Ledger HW implementation 

Below is an example implementation of how a Ledger hardware wallet `CryptoProvider` might implement the registration of its factory and how instantiation would work.

```go
// crypto/v2/providers/ledger/factory.go

const FACTORY_TYPE = "Ledger"

type LedgerCryptoProviderFactory struct {
    DevicePath string
    // Any other necessary fields goes here 
}

func (f *LedgerCryptoProviderFactory) CreateFromJson(data []byte) (CryptoProvider, error) {
    // Extract necessary data from the JSON to initialize a LedgerCryptoProvider
    if data == nil {
        return nil, fmt.Errorf("data is nil")
    }

    var config map[string]interface{}
    if err := json.Unmarshal(data, &config); err != nil {
        return nil, fmt.Errorf("failed to unmarshal JSON data: %v", err)
    }

    // Assuming the JSON contains necessary fields like devicePath
    devicePath, ok := config["devicePath"].(string)
    if !ok {
        return nil, fmt.Errorf("device path not found in JSON data")
    }

    // Initialize the LedgerCryptoProvider with the device path
    return &LedgerCryptoProvider{DevicePath: devicePath}, nil
}

func (f *LedgerCryptoProviderFactory) Type() string {
    return FACTORY_TYPE
}

// crypto/v2/examples/registerProvider.go

import (
    "crypto/v2/providers"
    "log"
)

func main() {
    // Initialize a new factory instance
    factory := NewFactory()
    
    // Create an instance of the ledger factory
    ledgerFactory := &ledger.LedgerCryptoProviderFactory{}

    // Register the factory
    factory.RegisterCryptoProviderFactory(ledgerFactory)

    // Example of loading a JSON file from a filesystem location
    // Assuming jsonFilePath already exists
    jsonData, err := ioutil.ReadFile(jsonFilePath)
    if err != nil {
        log.Fatalf("Error reading JSON file: %s", err)
    }

    ledgerProvider, err := factory.CreateCryptoProviderFromJson(jsonData)
    if err != nil {
        log.Fatalf("Error creating crypto provider from json: %s", err)
    }
    
    log.Printf("Provider from record created successfully: %+v", ledgerProvider.Metadata())

    // ledgerProvider CryptoProvider ready to use 
}
```

##### Especial use case: remote signers

It's important to note that the `CryptoProvider` interface is versatile enough to be implemented as a remote signer. This capability allows for the integration of remote cryptographic operations, which can be particularly useful in distributed or cloud-based environments where local cryptographic resources are limited or need to be managed centrally.

Here are a few of the services that can be leveraged:

* AWS CloudHSM
* Azure Key Vault
* HashiCorp Vault
* Google Cloud KMS

## Alternatives

It is important to note that all the code presented in this document is not in its final form and could be subject to changes at the time of implementation. The examples and implementations discussed should be interpreted as alternatives, providing a conceptual framework rather than definitive solutions. This flexibility allows for adjustments based on further insights, technical evaluations, or changing requirements as development progresses.

## Decision

We will:

* Refactor the module structure as described above.
* Define types and interfaces as the code attached.
* Refactor existing code into new structure and interfaces.
* Implement Unit Tests to ensure no backward compatibility issues.

> While an ADR is in the DRAFT or PROPOSED stage, this section should contain a
> summary of issues to be solved in future iterations (usually referencing comments
> from a pull-request discussion).
>
> Later, this section can optionally list ideas or improvements the author or
> reviewers found during the analysis of this ADR.

### Tentative Primitive Building Blocks

This is a **tentative** list of primitives that we might want to support.
**This is not a final list or comprehensive, and it is subject to change.**
Moreover, it is important to emphasize the purpose of this work allows extensibility so any other primitive can be added in the future.

* digital signatures
  * RSA (PSS)
  * ECDSA (secp256r1, secp256k1, etc.)
  * EdDSA (ed25519, ed448)
  * SR25519
  * Schnorr
  * Lattice-based (Dilithium)
  * BLS (BLS12-381, 377?)

* Hashing
  * sha2 / sha3
  * RIPEMD-160
  * blake2b,2s,3
  * Keccak-256 / shake256
  * bcrypt / scrypt / argon2, Argon2d/i/id
  * Pedersen