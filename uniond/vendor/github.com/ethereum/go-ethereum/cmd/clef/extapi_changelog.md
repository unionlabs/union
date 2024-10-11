## Changelog for external API

The API uses [semantic versioning](https://semver.org/).

TL;DR: Given a version number MAJOR.MINOR.PATCH, increment the:

* MAJOR version when you make incompatible API changes,
* MINOR version when you add functionality in a backwards-compatible manner, and
* PATCH version when you make backwards-compatible bug fixes.

Additional labels for pre-release and build metadata are available as extensions to the MAJOR.MINOR.PATCH format.

### 6.1.0

The API-method `account_signGnosisSafeTx` was added. This method takes two parameters, 
`[address, safeTx]`. The latter, `safeTx`, can be copy-pasted from the gnosis relay. For example: 

```
{
  "jsonrpc": "2.0",
  "method": "account_signGnosisSafeTx",
  "params": ["0xfd1c4226bfD1c436672092F4eCbfC270145b7256",
    {
      "safe": "0x25a6c4BBd32B2424A9c99aEB0584Ad12045382B3",
      "to": "0xB372a646f7F05Cc1785018dBDA7EBc734a2A20E2",
      "value": "20000000000000000",
      "data": null,
      "operation": 0,
      "gasToken": "0x0000000000000000000000000000000000000000",
      "safeTxGas": 27845,
      "baseGas": 0,
      "gasPrice": "0",
      "refundReceiver": "0x0000000000000000000000000000000000000000",
      "nonce": 2,
      "executionDate": null,
      "submissionDate": "2020-09-15T21:54:49.617634Z",
      "modified": "2020-09-15T21:54:49.617634Z",
      "blockNumber": null,
      "transactionHash": null,
      "safeTxHash": "0x2edfbd5bc113ff18c0631595db32eb17182872d88d9bf8ee4d8c2dd5db6d95e2",
      "executor": null,
      "isExecuted": false,
      "isSuccessful": null,
      "ethGasPrice": null,
      "gasUsed": null,
      "fee": null,
      "origin": null,
      "dataDecoded": null,
      "confirmationsRequired": null,
      "confirmations": [
        {
          "owner": "0xAd2e180019FCa9e55CADe76E4487F126Fd08DA34",
          "submissionDate": "2020-09-15T21:54:49.663299Z",
          "transactionHash": null,
          "confirmationType": "CONFIRMATION",
          "signature": "0x95a7250bb645f831c86defc847350e7faff815b2fb586282568e96cc859e39315876db20a2eed5f7a0412906ec5ab57652a6f645ad4833f345bda059b9da2b821c",
          "signatureType": "EOA"
        }
      ],
      "signatures": null
    }
  ],
  "id": 67
}
```

Not all fields are required, though. This method is really just a UX helper, which massages the 
input to conform to the `EIP-712` [specification](https://docs.gnosis.io/safe/docs/contracts_tx_execution/#transaction-hash) 
for the Gnosis Safe, and making the output be directly importable to by a relay service. 


### 6.0.0

* `New` was changed to deliver only an address, not the full `Account` data
* `Export` was moved from External API to the UI Server API

#### 5.0.0

* The external `account_EcRecover`-method was reimplemented.
* The external method `account_sign(address, data)` was replaced with `account_signData(contentType, address, data)`.
The addition of `contentType` makes it possible to use the method for different types of objects, such as:
  * signing data with an intended validator (not yet implemented)
  * signing clique headers,
  * signing plain personal messages,
* The external method `account_signTypedData` implements [EIP-712](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-712.md) and makes it possible to sign typed data.

#### 4.0.0

* The external `account_Ecrecover`-method was removed.
* The external `account_Import`-method was removed.

#### 3.0.0

* The external `account_List`-method was changed to not expose `url`, which contained info about the local filesystem. It now returns only a list of addresses.

#### 2.0.0

* Commit `73abaf04b1372fa4c43201fb1b8019fe6b0a6f8d`, move `from` into `transaction` object in `signTransaction`. This
makes the `accounts_signTransaction` identical to the old `eth_signTransaction`.


#### 1.0.0

Initial release.
