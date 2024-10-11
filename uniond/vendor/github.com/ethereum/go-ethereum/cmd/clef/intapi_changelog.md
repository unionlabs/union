## Changelog for internal API (ui-api)

The API uses [semantic versioning](https://semver.org/).

TL;DR: Given a version number MAJOR.MINOR.PATCH, increment the:

* MAJOR version when you make incompatible API changes,
* MINOR version when you add functionality in a backwards-compatible manner, and
* PATCH version when you make backwards-compatible bug fixes.

Additional labels for pre-release and build metadata are available as extensions to the MAJOR.MINOR.PATCH format.

### 7.0.1 

Added `clef_New` to the internal API callable from a UI.

> `New` creates a new password protected Account. The private key is protected with
> the given password. Users are responsible to backup the private key that is stored
> in the keystore location that was specified when this API was created.
> This method is the same as New on the external API, the difference being that
> this implementation does not ask for confirmation, since it's initiated by
> the user

### 7.0.0

- The `message` field was renamed to `messages` in all data signing request methods to better reflect that it's a list, not a value.
- The `storage.Put` and `storage.Get` methods in the rule execution engine were lower-cased to `storage.put` and `storage.get` to be consistent with JavaScript call conventions.

### 6.0.0

Removed `password` from responses to operations which require them. This is for two reasons,

- Consistency between how rulesets operate and how manual processing works. A rule can `Approve` but require the actual password to be stored in the clef storage.
With this change, the same stored password can be used even if rulesets are not enabled, but storage is.
- It also removes the usability-shortcut that a UI might otherwise want to implement; remembering passwords. Since we now will not require the
password on every `Approve`, there's no need for the UI to cache it locally.
  - In a future update, we'll likely add `clef_storePassword` to the internal API, so the user can store it via his UI (currently only CLI works).

Affected datatypes:
- `SignTxResponse`
- `SignDataResponse`
- `NewAccountResponse`

If `clef` requires a password, the `OnInputRequired` will be used to collect it.


### 5.0.0

Changed the namespace format to adhere to the legacy ethereum format: `name_methodName`. Changes:

* `ApproveTx` -> `ui_approveTx`
* `ApproveSignData` -> `ui_approveSignData`
* `ApproveExport` -> `removed`
* `ApproveImport`  -> `removed`
* `ApproveListing`  -> `ui_approveListing`
* `ApproveNewAccount`  -> `ui_approveNewAccount`
* `ShowError` -> `ui_showError`
* `ShowInfo` -> `ui_showInfo`
* `OnApprovedTx` -> `ui_onApprovedTx`
* `OnSignerStartup` -> `ui_onSignerStartup`
* `OnInputRequired` -> `ui_onInputRequired`


### 4.0.0

* Bidirectional communication implemented, so the UI can query `clef` via the stdin/stdout RPC channel. Methods implemented are:
  - `clef_listWallets`
  - `clef_listAccounts`
  - `clef_listWallets`
  - `clef_deriveAccount`
  - `clef_importRawKey`
  - `clef_openWallet`
  - `clef_chainId`
  - `clef_setChainId`
  - `clef_export`
  - `clef_import`

* The type `Account` was modified (the json-field `type` was removed), to consist of

```go
type Account struct {
	Address common.Address `json:"address"` // Ethereum account address derived from the key
	URL     URL            `json:"url"`     // Optional resource locator within a backend
}
```


### 3.2.0

* Make `ShowError`, `OnApprovedTx`, `OnSignerStartup` be json-rpc [notifications](https://www.jsonrpc.org/specification#notification):

> A Notification is a Request object without an "id" member. A Request object that is a Notification signifies the Client's lack of interest in the corresponding Response object, and as such no Response object needs to be returned to the client. The Server MUST NOT reply to a Notification, including those that are within a batch request.
>
>  Notifications are not confirmable by definition, since they do not have a Response object to be returned. As such, the Client would not be aware of any errors (like e.g. "Invalid params","Internal error"
### 3.1.0

* Add `ContentType` `string` to `SignDataRequest` to accommodate the latest EIP-191 and EIP-712 implementations.

### 3.0.0

* Make use of `OnInputRequired(info UserInputRequest)` for obtaining master password during startup

### 2.1.0

* Add `OnInputRequired(info UserInputRequest)` to internal API. This method is used when Clef needs user input, e.g. passwords.

The following structures are used:

```go
UserInputRequest struct {
	Prompt     string `json:"prompt"`
	Title      string `json:"title"`
	IsPassword bool   `json:"isPassword"`
}
UserInputResponse struct {
	Text string `json:"text"`
}
```

### 2.0.0

* Modify how `call_info` on a transaction is conveyed. New format:

```
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "ApproveTx",
  "params": [
    {
      "transaction": {
        "from": "0x82A2A876D39022B3019932D30Cd9c97ad5616813",
        "to": "0x07a565b7ed7d7a678680a4c162885bedbb695fe0",
        "gas": "0x333",
        "gasPrice": "0x123",
        "value": "0x10",
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
          "type": "WARNING",
          "message": "Tx contains data, but provided ABI signature could not be matched: Did not match: test (0 matches)"
        }
      ],
      "meta": {
        "remote": "127.0.0.1:54286",
        "local": "localhost:8550",
        "scheme": "HTTP/1.1"
      }
    }
  ]
}
```

#### 1.2.0

* Add `OnStartup` method, to provide the UI with information about what API version
the signer uses (both internal and external) as well as build-info and external api.

Example call:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "OnSignerStartup",
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

#### 1.1.0

* Add `OnApproved` method

#### 1.0.0

Initial release.
