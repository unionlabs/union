# Crypto

The `crypto` directory contains the components responsible for handling cryptographic operations, key management, and secure interactions with hardware wallets.

## Components

### Keyring

Keyring is the primary interface for managing cryptographic keys. It provides a unified API to create, store, retrieve, and manage keys securely across different storage backends.

#### Supported Backends

* **OS**: Uses the operating system's default credential store.
* **File**: Stores encrypted keyring in the application's configuration directory.
* **KWallet**: Integrates with KDE Wallet Manager.
* **Pass**: Leverages the `pass` command-line utility.
* **Keyctl**: Uses Linux's kernel security key management system.
* **Test**: Stores (insecurely) keys to disk for testing purposes.
* **Memory**: Provides transient storage where keys are discarded when the process terminates.

### Codec

The Codec component handles serialization and deserialization of cryptographic structures in the crypto package. It ensures proper encoding of keys, signatures, and other artifacts for storage and transmission. The Codec also manages conversion between CometBFT and SDK key formats.

### Ledger Integration

Support for Ledger hardware wallets is integrated to provide enhanced security for key management and signing operations. The Ledger integration supports SECP256K1 keys and offers various features:

#### Key Features

* **Public Key Retrieval**: Supports both safe (with user verification) and unsafe (without user verification) methods to retrieve public keys from the Ledger device.
* **Address Generation**: Can generate and display Bech32 addresses on the Ledger device for user verification.
* **Transaction Signing**: Allows signing of transactions with user confirmation on the Ledger device.
* **Multiple HD Path Support**: Supports various BIP44 derivation paths for key generation and management.
* **Customizable Options**: Provides options to customize Ledger usage, including app name, public key creation, and DER to BER signature conversion.

#### Implementation Details

* The integration is built to work with or without CGO.
* It includes a mock implementation for testing purposes, which can be enabled with the `test_ledger_mock` build tag.
* The real Ledger device interaction is implemented when the `ledger` build tag is used.
* The integration supports both SIGN_MODE_LEGACY_AMINO_JSON and SIGN_MODE_TEXTUAL signing modes.

#### Usage Considerations

* Ledger support requires the appropriate Cosmos app to be installed and opened on the Ledger device.
* The integration includes safeguards to prevent key overwriting and ensures that the correct device and app are being used.

#### Security Notes

* The integration includes methods to validate keys and addresses with user confirmation on the Ledger device.
* It's recommended to use the safe methods that require user verification for critical operations like key generation and address display.
* The mock implementation should only be used for testing and development purposes, not in production environments.