# Clef

Clef can be used to sign transactions and data and is meant as a(n eventual) replacement for Geth's account management. This allows DApps to not depend on Geth's account management. When a DApp wants to sign data (or a transaction), it can send the content to Clef, which will then provide the user with context and asks for permission to sign the content. If the users grants the signing request, Clef will send the signature back to the DApp.

This setup allows a DApp to connect to a remote Ethereum node and send transactions that are locally signed. This can help in situations when a DApp is connected to an untrusted remote Ethereum node, because a local one is not available, not synchronised with the chain, or is a node that has no built-in (or limited) account management.

Clef can run as a daemon on the same machine, off a usb-stick like [USB armory](https://inversepath.com/usbarmory), or even a separate VM in a [QubesOS](https://www.qubes-os.org/) type setup.

Check out the

* [CLI tutorial](tutorial.md) for some concrete examples on how Clef works.
* [Setup docs](docs/setup.md) for information on how to configure Clef on QubesOS or USB Armory.
* [Data types](datatypes.md) for details on the communication messages between Clef and an external UI.

## Command line flags

Clef accepts the following command line options:

```
COMMANDS:
   init    Initialize the signer, generate secret storage
   attest  Attest that a js-file is to be used
   setpw   Store a credential for a keystore file
   delpw   Remove a credential for a keystore file
   gendoc  Generate documentation about json-rpc format
   help    Shows a list of commands or help for one command

GLOBAL OPTIONS:
   --loglevel value        log level to emit to the screen (default: 4)
   --keystore value        Directory for the keystore (default: "$HOME/.ethereum/keystore")
   --configdir value       Directory for Clef configuration (default: "$HOME/.clef")
   --chainid value         Chain id to use for signing (1=mainnet, 5=Goerli) (default: 1)
   --lightkdf              Reduce key-derivation RAM & CPU usage at some expense of KDF strength
   --nousb                 Disables monitoring for and managing USB hardware wallets
   --pcscdpath value       Path to the smartcard daemon (pcscd) socket file (default: "/run/pcscd/pcscd.comm")
   --http.addr value       HTTP-RPC server listening interface (default: "localhost")
   --http.vhosts value     Comma separated list of virtual hostnames from which to accept requests (server enforced). Accepts '*' wildcard. (default: "localhost")
   --ipcdisable            Disable the IPC-RPC server
   --ipcpath               Filename for IPC socket/pipe within the datadir (explicit paths escape it)
   --http                  Enable the HTTP-RPC server
   --http.port value       HTTP-RPC server listening port (default: 8550)
   --signersecret value    A file containing the (encrypted) master seed to encrypt Clef data, e.g. keystore credentials and ruleset hash
   --4bytedb-custom value  File used for writing new 4byte-identifiers submitted via API (default: "./4byte-custom.json")
   --auditlog value        File used to emit audit logs. Set to "" to disable (default: "audit.log")
   --rules value           Path to the rule file to auto-authorize requests with
   --stdio-ui              Use STDIN/STDOUT as a channel for an external UI. This means that an STDIN/STDOUT is used for RPC-communication with a e.g. a graphical user interface, and can be used when Clef is started by an external process.
   --stdio-ui-test         Mechanism to test interface between Clef and UI. Requires 'stdio-ui'.
   --advanced              If enabled, issues warnings instead of rejections for suspicious requests. Default off
   --suppress-bootwarn     If set, does not show the warning during boot
   --help, -h              show help
   --version, -v           print the version
```

Example:

```
$ clef -keystore /my/keystore -chainid 4
```

## Security model

The security model of Clef is as follows:

* One critical component (the Clef binary / daemon) is responsible for handling cryptographic operations: signing, private keys, encryption/decryption of keystore files.
* Clef has a well-defined 'external' API.
* The 'external' API is considered UNTRUSTED.
* Clef also communicates with whatever process that invoked the binary, via stdin/stdout.
  * This channel is considered 'trusted'. Over this channel, approvals and passwords are communicated.

The general flow for signing a transaction using e.g. Geth is as follows:
![image](sign_flow.png)

In this case, `geth` would be started with `--signer http://localhost:8550` and would relay requests to `eth.sendTransaction`.

## TODOs

Some snags and todos

* [ ] Clef should take a startup param "--no-change", for UIs that do not contain the capability to perform changes to things, only approve/deny. Such a UI should be able to start the signer in a more secure mode by telling it that it only wants approve/deny capabilities.
* [x] It would be nice if Clef could collect new 4byte-id:s/method selectors, and have a secondary database for those (`4byte_custom.json`). Users could then (optionally) submit their collections for inclusion upstream.
* [ ] It should be possible to configure Clef to check if an account is indeed known to it, before passing on to the UI. The reason it currently does not, is that it would make it possible to enumerate accounts if it immediately returned "unknown account" (side channel attack).
* [x] It should be possible to configure Clef to auto-allow listing (certain) accounts, instead of asking every time.
* [x] Done Upon startup, Clef should spit out some info to the caller (particularly important when executed in `stdio-ui`-mode), invoking methods with the following info:
  * [x] Version info about the signer
  * [x] Address of API (HTTP/IPC)
  * [ ] List of known accounts
* [ ] Have a default timeout on signing operations, so that if the user has not answered within e.g. 60 seconds, the request is rejected.
* [ ] `account_signRawTransaction`
* [ ] `account_bulkSignTransactions([] transactions)` should
   * only exist if enabled via config/flag
   * only allow non-data-sending transactions
   * all txs must use the same `from`-account
   * let the user confirm, showing
      * the total amount
      * the number of unique recipients

* Geth todos
    - The signer should pass the `Origin` header as call-info to the UI. As of right now, the way that info about the request is put together is a bit of a hack into the HTTP server. This could probably be greatly improved.
    - Relay: Geth should be started in `geth --signer localhost:8550`.
    - Currently, the Geth APIs use `common.Address` in the arguments to transaction submission (e.g `to` field). This type is 20 `bytes`, and is incapable of carrying checksum information. The signer uses `common.MixedcaseAddress`, which retains the original input.
    - The Geth API should switch to use the same type, and relay `to`-account verbatim to the external API.
* [x] Storage
    * [x] An encrypted key-value storage should be implemented.
    * See [rules.md](rules.md) for more info about this.
* Another potential thing to introduce is pairing.
  * To prevent spurious requests which users just accept, implement a way to "pair" the caller with the signer (external API).
  * Thus Geth/cpp would cryptographically handshake and afterwards the caller would be allowed to make signing requests.
  * This feature would make the addition of rules less dangerous.

* Wallets / accounts. Add API methods for wallets.

## Communication

### External API

Clef listens to HTTP requests on `http.addr`:`http.port` (or to IPC on `ipcpath`), with the same JSON-RPC standard as Geth. The messages are expected to be [JSON-RPC 2.0 standard](https://www.jsonrpc.org/specification).

Some of these calls can require user interaction. Clients must be aware that responses may be delayed significantly or may never be received if a user decides to ignore the confirmation request.

The External API is **untrusted**: it does not accept credentials, nor does it expect that requests have any authority.

### Internal UI API

Clef has one native console-based UI, for operation without any standalone tools. However, there is also an API to communicate with an external UI. To enable that UI, the signer needs to be executed with the `--stdio-ui` option, which allocates `stdin` / `stdout` for the UI API.

An example (insecure) proof-of-concept of has been implemented in `pythonsigner.py`.

The model is as follows:

* The user starts the UI app (`pythonsigner.py`).
* The UI app starts `clef` with `--stdio-ui`, and listens to the
process output for confirmation-requests.
* `clef` opens the external HTTP API.
* When the `signer` receives requests, it sends a JSON-RPC request via `stdout`.
* The UI app prompts the user accordingly, and responds to `clef`.
* `clef` signs (or not), and responds to the original request.

## External API

See the [external API changelog](extapi_changelog.md) for information about changes to this API.

### Encoding
- number: positive integers that are hex encoded
- data: hex encoded data
- string: ASCII string

All hex encoded values must be prefixed with `0x`.

### account_new

#### Create new password protected account

The signer will generate a new private key, encrypt it according to [web3 keystore spec](https://github.com/ethereum/wiki/wiki/Web3-Secret-Storage-Definition) and store it in the keystore directory.  
The client is responsible for creating a backup of the keystore. If the keystore is lost there is no method of retrieving lost accounts.

#### Arguments

None

#### Result
  - address [string]: account address that is derived from the generated key

#### Sample call
```json
{
  "id": 0,
  "jsonrpc": "2.0",
  "method": "account_new",
  "params": []
}
```
Response
```json
{
  "id": 0,
  "jsonrpc": "2.0",
  "result": "0xbea9183f8f4f03d427f6bcea17388bdff1cab133"
}
```

### account_list

#### List available accounts
   List all accounts that this signer currently manages

#### Arguments

None

#### Result
  - array with account records:
     - account.address [string]: account address that is derived from the generated key

#### Sample call
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "account_list"
}
```
Response
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "result": [
    "0xafb2f771f58513609765698f65d3f2f0224a956f",
    "0xbea9183f8f4f03d427f6bcea17388bdff1cab133"
  ]
}
```

### account_signTransaction

#### Sign transactions
   Signs a transaction and responds with the signed transaction in RLP-encoded and JSON forms.

#### Arguments
  1. transaction object:
     - `from` [address]: account to send the transaction from
     - `to` [address]: receiver account. If omitted or `0x`, will cause contract creation.
     - `gas` [number]: maximum amount of gas to burn
     - `gasPrice` [number]: gas price
     - `value` [number:optional]: amount of Wei to send with the transaction
     - `data` [data:optional]:  input data
     - `nonce` [number]: account nonce
  1. method signature [string:optional]
     - The method signature, if present, is to aid decoding the calldata. Should consist of `methodname(paramtype,...)`, e.g. `transfer(uint256,address)`. The signer may use this data to parse the supplied calldata, and show the user. The data, however, is considered totally untrusted, and reliability is not expected.


#### Result
  - raw [data]: signed transaction in RLP encoded form
  - tx [json]: signed transaction in JSON form

#### Sample call
```json
{
  "id": 2,
  "jsonrpc": "2.0",
  "method": "account_signTransaction",
  "params": [
    {
      "from": "0x1923f626bb8dc025849e00f99c25fe2b2f7fb0db",
      "gas": "0x55555",
      "gasPrice": "0x1234",
      "input": "0xabcd",
      "nonce": "0x0",
      "to": "0x07a565b7ed7d7a678680a4c162885bedbb695fe0",
      "value": "0x1234"
    }
  ]
}
```
Response

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "raw": "0xf88380018203339407a565b7ed7d7a678680a4c162885bedbb695fe080a44401a6e4000000000000000000000000000000000000000000000000000000000000001226a0223a7c9bcf5531c99be5ea7082183816eb20cfe0bbc322e97cc5c7f71ab8b20ea02aadee6b34b45bb15bc42d9c09de4a6754e7000908da72d48cc7704971491663",
    "tx": {
      "nonce": "0x0",
      "gasPrice": "0x1234",
      "gas": "0x55555",
      "to": "0x07a565b7ed7d7a678680a4c162885bedbb695fe0",
      "value": "0x1234",
      "input": "0xabcd",
      "v": "0x26",
      "r": "0x223a7c9bcf5531c99be5ea7082183816eb20cfe0bbc322e97cc5c7f71ab8b20e",
      "s": "0x2aadee6b34b45bb15bc42d9c09de4a6754e7000908da72d48cc7704971491663",
      "hash": "0xeba2df809e7a612a0a0d444ccfa5c839624bdc00dd29e3340d46df3870f8a30e"
    }
  }
}
```
#### Sample call with ABI-data


```json
{
  "id": 67,
  "jsonrpc": "2.0",
  "method": "account_signTransaction",
  "params": [
    {
      "from": "0x694267f14675d7e1b9494fd8d72fefe1755710fa",
      "gas": "0x333",
      "gasPrice": "0x1",
      "nonce": "0x0",
      "to": "0x07a565b7ed7d7a678680a4c162885bedbb695fe0",
      "value": "0x0",
      "data": "0x4401a6e40000000000000000000000000000000000000000000000000000000000000012"
    },
    "safeSend(address)"
  ]
}
```
Response

```json
{
  "jsonrpc": "2.0",
  "id": 67,
  "result": {
    "raw": "0xf88380018203339407a565b7ed7d7a678680a4c162885bedbb695fe080a44401a6e4000000000000000000000000000000000000000000000000000000000000001226a0223a7c9bcf5531c99be5ea7082183816eb20cfe0bbc322e97cc5c7f71ab8b20ea02aadee6b34b45bb15bc42d9c09de4a6754e7000908da72d48cc7704971491663",
    "tx": {
      "nonce": "0x0",
      "gasPrice": "0x1",
      "gas": "0x333",
      "to": "0x07a565b7ed7d7a678680a4c162885bedbb695fe0",
      "value": "0x0",
      "input": "0x4401a6e40000000000000000000000000000000000000000000000000000000000000012",
      "v": "0x26",
      "r": "0x223a7c9bcf5531c99be5ea7082183816eb20cfe0bbc322e97cc5c7f71ab8b20e",
      "s": "0x2aadee6b34b45bb15bc42d9c09de4a6754e7000908da72d48cc7704971491663",
      "hash": "0xeba2df809e7a612a0a0d444ccfa5c839624bdc00dd29e3340d46df3870f8a30e"
    }
  }
}
```

Bash example:
```bash
> curl -H "Content-Type: application/json" -X POST --data '{"jsonrpc":"2.0","method":"account_signTransaction","params":[{"from":"0x694267f14675d7e1b9494fd8d72fefe1755710fa","gas":"0x333","gasPrice":"0x1","nonce":"0x0","to":"0x07a565b7ed7d7a678680a4c162885bedbb695fe0", "value":"0x0", "data":"0x4401a6e40000000000000000000000000000000000000000000000000000000000000012"},"safeSend(address)"],"id":67}' http://localhost:8550/

{"jsonrpc":"2.0","id":67,"result":{"raw":"0xf88380018203339407a565b7ed7d7a678680a4c162885bedbb695fe080a44401a6e4000000000000000000000000000000000000000000000000000000000000001226a0223a7c9bcf5531c99be5ea7082183816eb20cfe0bbc322e97cc5c7f71ab8b20ea02aadee6b34b45bb15bc42d9c09de4a6754e7000908da72d48cc7704971491663","tx":{"nonce":"0x0","gasPrice":"0x1","gas":"0x333","to":"0x07a565b7ed7d7a678680a4c162885bedbb695fe0","value":"0x0","input":"0x4401a6e40000000000000000000000000000000000000000000000000000000000000012","v":"0x26","r":"0x223a7c9bcf5531c99be5ea7082183816eb20cfe0bbc322e97cc5c7f71ab8b20e","s":"0x2aadee6b34b45bb15bc42d9c09de4a6754e7000908da72d48cc7704971491663","hash":"0xeba2df809e7a612a0a0d444ccfa5c839624bdc00dd29e3340d46df3870f8a30e"}}}
```

### account_signData

#### Sign data
   Signs a chunk of data and returns the calculated signature.

#### Arguments
  - content type [string]: type of signed data
     - `text/validator`: hex data with custom validator defined in a contract
     - `application/clique`: [clique](https://github.com/ethereum/EIPs/issues/225) headers
     - `text/plain`: simple hex data validated by `account_ecRecover`
  - account [address]: account to sign with
  - data [object]: data to sign

#### Result
  - calculated signature [data]

#### Sample call
```json
{
  "id": 3,
  "jsonrpc": "2.0",
  "method": "account_signData",
  "params": [
    "data/plain",
    "0x1923f626bb8dc025849e00f99c25fe2b2f7fb0db",
    "0xaabbccdd"
  ]
}
```
Response

```json
{
  "id": 3,
  "jsonrpc": "2.0",
  "result": "0x5b6693f153b48ec1c706ba4169960386dbaa6903e249cc79a8e6ddc434451d417e1e57327872c7f538beeb323c300afa9999a3d4a5de6caf3be0d5ef832b67ef1c"
}
```

### account_signTypedData

#### Sign data
   Signs a chunk of structured data conformant to [EIP-712](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-712.md) and returns the calculated signature.

#### Arguments
  - account [address]: account to sign with
  - data [object]: data to sign

#### Result
  - calculated signature [data]

#### Sample call
```json
{
  "id": 68,
  "jsonrpc": "2.0",
  "method": "account_signTypedData",
  "params": [
    "0xcd2a3d9f938e13cd947ec05abc7fe734df8dd826",
    {
      "types": {
        "EIP712Domain": [
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "version",
            "type": "string"
          },
          {
            "name": "chainId",
            "type": "uint256"
          },
          {
            "name": "verifyingContract",
            "type": "address"
          }
        ],
        "Person": [
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "wallet",
            "type": "address"
          }
        ],
        "Mail": [
          {
            "name": "from",
            "type": "Person"
          },
          {
            "name": "to",
            "type": "Person"
          },
          {
            "name": "contents",
            "type": "string"
          }
        ]
      },
      "primaryType": "Mail",
      "domain": {
        "name": "Ether Mail",
        "version": "1",
        "chainId": 1,
        "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
      },
      "message": {
        "from": {
          "name": "Cow",
          "wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
        },
        "to": {
          "name": "Bob",
          "wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
        },
        "contents": "Hello, Bob!"
      }
    }
  ]
}
```
Response

```json
{
    "id": 1,
    "jsonrpc": "2.0",
    "result": "0x4355c47d63924e8a72e509b65029052eb6c299d53a04e167c5775fd466751c9d07299936d304c153f6443dfa05f40ff007d72911b6f72307f996231605b915621c"
}
```

### account_ecRecover

#### Recover the signing address

Derive the address from the account that was used to sign data with content type `text/plain` and the signature.

#### Arguments
  - data [data]: data that was signed
  - signature [data]: the signature to verify

#### Result
  - derived account [address]

#### Sample call
```json
{
  "id": 4,
  "jsonrpc": "2.0",
  "method": "account_ecRecover",
  "params": [
    "0xaabbccdd",
    "0x5b6693f153b48ec1c706ba4169960386dbaa6903e249cc79a8e6ddc434451d417e1e57327872c7f538beeb323c300afa9999a3d4a5de6caf3be0d5ef832b67ef1c"
  ]
}
```
Response

```json
{
  "id": 4,
  "jsonrpc": "2.0",
  "result": "0x1923f626bb8dc025849e00f99c25fe2b2f7fb0db"
}
```

### account_version

#### Get external API version

Get the version of the external API used by Clef.

#### Arguments

None

#### Result

* external API version [string]

#### Sample call
```json
{
  "id": 0,
  "jsonrpc": "2.0",
  "method": "account_version",
  "params": []
}
```

Response
```json
{
    "id": 0,
    "jsonrpc": "2.0",
    "result": "6.0.0"
}
```

## UI API

These methods needs to be implemented by a UI listener.

By starting the signer with the switch `--stdio-ui-test`, the signer will invoke all known methods, and expect the UI to respond with
denials. This can be used during development to ensure that the API is (at least somewhat) correctly implemented.
See `pythonsigner`, which can be invoked via `python3 pythonsigner.py test` to perform the 'denial-handshake-test'.

All methods in this API use object-based parameters, so that there can be no mixup of parameters: each piece of data is accessed by key.

See the [ui API changelog](intapi_changelog.md) for information about changes to this API.

OBS! A slight deviation from `json` standard is in place: every request and response should be confined to a single line.
Whereas the `json` specification allows for linebreaks, linebreaks __should not__ be used in this communication channel, to make
things simpler for both parties.

### ApproveTx / `ui_approveTx`

Invoked when there's a transaction for approval.


#### Sample call

Here's a method invocation:
```bash

curl -i -H "Content-Type: application/json" -X POST --data '{"jsonrpc":"2.0","method":"account_signTransaction","params":[{"from":"0x694267f14675d7e1b9494fd8d72fefe1755710fa","gas":"0x333","gasPrice":"0x1","nonce":"0x0","to":"0x07a565b7ed7d7a678680a4c162885bedbb695fe0", "value":"0x0", "data":"0x4401a6e40000000000000000000000000000000000000000000000000000000000000012"},"safeSend(address)"],"id":67}' http://localhost:8550/
```
Results in the following invocation on the UI:
```json

{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "ui_approveTx",
  "params": [
    {
      "transaction": {
        "from": "0x0x694267f14675d7e1b9494fd8d72fefe1755710fa",
        "to": "0x0x07a565b7ed7d7a678680a4c162885bedbb695fe0",
        "gas": "0x333",
        "gasPrice": "0x1",
        "value": "0x0",
        "nonce": "0x0",
        "data": "0x4401a6e40000000000000000000000000000000000000000000000000000000000000012",
        "input": null
      },
      "call_info": [
          {
            "type": "WARNING",
            "message": "Invalid checksum on to-address"
          },
          {
            "type": "Info",
            "message": "safeSend(address: 0x0000000000000000000000000000000000000012)"
          }
        ],
      "meta": {
        "remote": "127.0.0.1:48486",
        "local": "localhost:8550",
        "scheme": "HTTP/1.1"
      }
    }
  ]
}

```

The same method invocation, but with invalid data:
```bash

curl -i -H "Content-Type: application/json" -X POST --data '{"jsonrpc":"2.0","method":"account_signTransaction","params":[{"from":"0x694267f14675d7e1b9494fd8d72fefe1755710fa","gas":"0x333","gasPrice":"0x1","nonce":"0x0","to":"0x07a565b7ed7d7a678680a4c162885bedbb695fe0", "value":"0x0", "data":"0x4401a6e40000000000000002000000000000000000000000000000000000000000000012"},"safeSend(address)"],"id":67}' http://localhost:8550/
```

```json

{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "ui_approveTx",
  "params": [
    {
      "transaction": {
        "from": "0x0x694267f14675d7e1b9494fd8d72fefe1755710fa",
        "to": "0x0x07a565b7ed7d7a678680a4c162885bedbb695fe0",
        "gas": "0x333",
        "gasPrice": "0x1",
        "value": "0x0",
        "nonce": "0x0",
        "data": "0x4401a6e40000000000000002000000000000000000000000000000000000000000000012",
        "input": null
      },
      "call_info": [
          {
            "type": "WARNING",
            "message": "Invalid checksum on to-address"
          },
          {
            "type": "WARNING",
            "message": "Transaction data did not match ABI-interface: WARNING: Supplied data is stuffed with extra data. \nWant 0000000000000002000000000000000000000000000000000000000000000012\nHave 0000000000000000000000000000000000000000000000000000000000000012\nfor method safeSend(address)"
          }
        ],
      "meta": {
        "remote": "127.0.0.1:48492",
        "local": "localhost:8550",
        "scheme": "HTTP/1.1"
      }
    }
  ]
}


```

One which has missing `to`, but with no `data`:


```json

{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "ui_approveTx",
  "params": [
    {
      "transaction": {
        "from": "",
        "to": null,
        "gas": "0x0",
        "gasPrice": "0x0",
        "value": "0x0",
        "nonce": "0x0",
        "data": null,
        "input": null
      },
      "call_info": [
          {
            "type": "CRITICAL",
            "message": "Tx will create contract with empty code!"
          }
        ],
      "meta": {
        "remote": "signer binary",
        "local": "main",
        "scheme": "in-proc"
      }
    }
  ]
}
```

### ApproveListing / `ui_approveListing`

Invoked when a request for account listing has been made.

#### Sample call

```json

{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "ui_approveListing",
  "params": [
    {
      "accounts": [
        {
          "url": "keystore:///home/bazonk/.ethereum/keystore/UTC--2017-11-20T14-44-54.089682944Z--123409812340981234098123409812deadbeef42",
          "address": "0x123409812340981234098123409812deadbeef42"
        },
        {
          "url": "keystore:///home/bazonk/.ethereum/keystore/UTC--2017-11-23T21-59-03.199240693Z--cafebabedeadbeef34098123409812deadbeef42",
          "address": "0xcafebabedeadbeef34098123409812deadbeef42"
        }
      ],
      "meta": {
        "remote": "signer binary",
        "local": "main",
        "scheme": "in-proc"
      }
    }
  ]
}

```


### ApproveSignData / `ui_approveSignData`

#### Sample call

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "ui_approveSignData",
  "params": [
    {
      "address": "0x123409812340981234098123409812deadbeef42",
      "raw_data": "0x01020304",
      "messages": [
        {
          "name": "message",
          "value": "\u0019Ethereum Signed Message:\n4\u0001\u0002\u0003\u0004",
          "type": "text/plain"
        }
      ],
      "hash": "0x7e3a4e7a9d1744bc5c675c25e1234ca8ed9162bd17f78b9085e48047c15ac310",
      "meta": {
        "remote": "signer binary",
        "local": "main",
        "scheme": "in-proc"
      }
    }
  ]
}
```

### ApproveNewAccount / `ui_approveNewAccount`

Invoked when a request for creating a new account has been made.

#### Sample call

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "ui_approveNewAccount",
  "params": [
    {
      "meta": {
        "remote": "signer binary",
        "local": "main",
        "scheme": "in-proc"
      }
    }
  ]
}
```

### ShowInfo / `ui_showInfo`

The UI should show the info (a single message) to the user. Does not expect response.

#### Sample call

```json
{
  "jsonrpc": "2.0",
  "id": 9,
  "method": "ui_showInfo",
  "params": [
    "Tests completed"
  ]
}

```

### ShowError / `ui_showError`

The UI should show the error (a single message) to the user. Does not expect response.

```json

{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "ui_showError",
  "params": [
    "Something bad happened!"
  ]
}

```

### OnApprovedTx / `ui_onApprovedTx`

`OnApprovedTx` is called when a transaction has been approved and signed. The call contains the return value that will be sent to the external caller.  The return value from this method is ignored - the reason for having this callback is to allow the ruleset to keep track of approved transactions.

When implementing rate-limited rules, this callback should be used.

TLDR; Use this method to keep track of signed transactions, instead of using the data in `ApproveTx`.

Example call:
```json

{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "ui_onApprovedTx",
  "params": [
    {
      "raw": "0xf88380018203339407a565b7ed7d7a678680a4c162885bedbb695fe080a44401a6e4000000000000000000000000000000000000000000000000000000000000001226a0223a7c9bcf5531c99be5ea7082183816eb20cfe0bbc322e97cc5c7f71ab8b20ea02aadee6b34b45bb15bc42d9c09de4a6754e7000908da72d48cc7704971491663",
      "tx": {
        "nonce": "0x0",
        "gasPrice": "0x1",
        "gas": "0x333",
        "to": "0x07a565b7ed7d7a678680a4c162885bedbb695fe0",
        "value": "0x0",
        "input": "0x4401a6e40000000000000000000000000000000000000000000000000000000000000012",
        "v": "0x26",
        "r": "0x223a7c9bcf5531c99be5ea7082183816eb20cfe0bbc322e97cc5c7f71ab8b20e",
        "s": "0x2aadee6b34b45bb15bc42d9c09de4a6754e7000908da72d48cc7704971491663",
        "hash": "0xeba2df809e7a612a0a0d444ccfa5c839624bdc00dd29e3340d46df3870f8a30e"
      }
    }
  ]
}
```

### OnSignerStartup / `ui_onSignerStartup`

This method provides the UI with information about what API version the signer uses (both internal and external) as well as build-info and external API,
in k/v-form.

Example call:
```json

{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "ui_onSignerStartup",
  "params": [
    {
      "info": {
        "extapi_http": "http://localhost:8550",
        "extapi_ipc": null,
        "extapi_version": "2.0.0",
        "intapi_version": "1.2.0"
      }
    }
  ]
}

```

### OnInputRequired / `ui_onInputRequired`

Invoked when Clef requires user input (e.g. a password).

Example call:
```json

{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "ui_onInputRequired",
  "params": [
    {
      "title": "Account password",
      "prompt": "Please enter the password for account 0x694267f14675d7e1b9494fd8d72fefe1755710fa",
      "isPassword": true
    }
  ]
}
```


### Rules for UI apis

A UI should conform to the following rules.

* A UI MUST NOT load any external resources that were not embedded/part of the UI package.
  * For example, not load icons, stylesheets from the internet
  * Not load files from the filesystem, unless they reside in the same local directory (e.g. config files)
* A Graphical UI MUST show the blocky-identicon for ethereum addresses.
* A UI MUST warn display appropriate warning if the destination-account is formatted with invalid checksum.
* A UI MUST NOT open any ports or services
  * The signer opens the public port
* A UI SHOULD verify the permissions on the signer binary, and refuse to execute or warn if permissions allow non-user write.
* A UI SHOULD inform the user about the `SHA256` or `MD5` hash of the binary being executed
* A UI SHOULD NOT maintain a secondary storage of data, e.g. list of accounts
  * The signer provides accounts
* A UI SHOULD, to the best extent possible, use static linking / bundling, so that required libraries are bundled
along with the UI.


### UI Implementations

There are a couple of implementation for a UI. We'll try to keep this list up to date.

| Name | Repo | UI type| No external resources| Blocky support| Verifies permissions | Hash information | No secondary storage | Statically linked| Can modify parameters|
| ---- | ---- | -------| ---- | ---- | ---- |---- | ---- | ---- | ---- |
| QtSigner| https://github.com/holiman/qtsigner/| Python3/QT-based| :+1:| :+1:| :+1:| :+1:| :+1:| :x: |  :+1: (partially)|
| GtkSigner| https://github.com/holiman/gtksigner| Python3/GTK-based| :+1:| :x:| :x:| :+1:| :+1:| :x: |  :x: |
| Frame | https://github.com/floating/frame/commits/go-signer| Electron-based| :x:| :x:| :x:| :x:| ?| :x: |  :x: |
| Clef UI| https://github.com/ethereum/clef-ui| Golang/QT-based| :+1:| :+1:| :x:| :+1:| :+1:| :x: |  :+1: (approve tx only)|
