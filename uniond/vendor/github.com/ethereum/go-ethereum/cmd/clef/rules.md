# Rules

The `signer` binary contains a ruleset engine, implemented with [OttoVM](https://github.com/robertkrimen/otto)

It enables usecases like the following:

* I want to auto-approve transactions with contract `CasinoDapp`, with up to `0.05 ether` in value to maximum `1 ether` per 24h period
* I want to auto-approve transaction to contract `EthAlarmClock` with `data`=`0xdeadbeef`, if `value=0`, `gas < 44k` and `gasPrice < 40Gwei`

The two main features that are required for this to work well are;

1. Rule Implementation: how to create, manage and interpret rules in a flexible but secure manner
2. Credential managements and credentials; how to provide auto-unlock without exposing keys unnecessarily.

The section below deals with both of them

## Rule Implementation

A ruleset file is implemented as a `js` file. Under the hood, the ruleset-engine is a `SignerUI`, implementing the same methods as the `json-rpc` methods
defined in the UI protocol. Example:

```js
function asBig(str) {
	if (str.slice(0, 2) == "0x") {
		return new BigNumber(str.slice(2), 16)
	}
	return new BigNumber(str)
}

// Approve transactions to a certain contract if value is below a certain limit
function ApproveTx(req) {
	var limit = big.Newint("0xb1a2bc2ec50000")
	var value = asBig(req.transaction.value);

	if (req.transaction.to.toLowerCase() == "0xae967917c465db8578ca9024c205720b1a3651a9") && value.lt(limit)) {
		return "Approve"
	}
	// If we return "Reject", it will be rejected.
	// By not returning anything, it will be passed to the next UI, for manual processing
}

// Approve listings if request made from IPC
function ApproveListing(req){
    if (req.metadata.scheme == "ipc"){ return "Approve"}
}
```

Whenever the external API is called (and the ruleset is enabled), the `signer` calls the UI, which is an instance of a ruleset-engine. The ruleset-engine
invokes the corresponding method. In doing so, there are three possible outcomes:

1. JS returns "Approve"
  * Auto-approve request
2. JS returns "Reject"
  * Auto-reject request
3. Error occurs, or something else is returned
  * Pass on to `next` ui: the regular UI channel.

A more advanced example can be found below, "Example 1: ruleset for a rate-limited window", using `storage` to `Put` and `Get` `string`s by key.

* At the time of writing, storage only exists as an ephemeral unencrypted implementation, to be used during testing.

### Things to note

The Otto vm has a few [caveats](https://github.com/robertkrimen/otto):

* "use strict" will parse, but does nothing.
* The regular expression engine (re2/regexp) is not fully compatible with the ECMA5 specification.
* Otto targets ES5. ES6 features (eg: Typed Arrays) are not supported.

Additionally, a few more have been added

* The rule execution cannot load external javascript files.
* The only preloaded library is [`bignumber.js`](https://github.com/MikeMcl/bignumber.js) version `2.0.3`. This one is fairly old, and is not aligned with the documentation at the github repository.
* Each invocation is made in a fresh virtual machine. This means that you cannot store data in global variables between invocations. This is a deliberate choice -- if you want to store data, use the disk-backed `storage`, since rules should not rely on ephemeral data.
* Javascript API parameters are _always_ an object. This is also a design choice, to ensure that parameters are accessed by _key_ and not by order. This is to prevent mistakes due to missing parameters or parameter changes.
* The JS engine has access to `storage` and `console`.

#### Security considerations

##### Security of ruleset

Some security precautions can be made, such as:

* Never load `ruleset.js` unless the file is `readonly` (`r-??-??-?`). If the user wishes to modify the ruleset, he must make it writeable and then set back to readonly.
  * This is to prevent attacks where files are dropped on the users disk.
* Since we're going to have to have some form of secure storage (not defined in this section), we could also store the `sha3` of the `ruleset.js` file in there.
  * If the user wishes to modify the ruleset, he'd then have to perform e.g. `signer --attest /path/to/ruleset --credential <creds>`

##### Security of implementation

The drawbacks of this very flexible solution is that the `signer` needs to contain a javascript engine. This is pretty simple to implement, since it's already
implemented for `geth`. There are no known security vulnerabilities in, nor have we had any security-problems with it so far.

The javascript engine would be an added attack surface; but if the validation of `rulesets` is made good (with hash-based attestation), the actual javascript cannot be considered
an attack surface -- if an attacker can control the ruleset, a much simpler attack would be to implement an "always-approve" rule instead of exploiting the js vm. The only benefit
to be gained from attacking the actual `signer` process from the `js` side would be if it could somehow extract cryptographic keys from memory.

##### Security in usability

Javascript is flexible, but also easy to get wrong, especially when users assume that `js` can handle large integers natively. Typical errors
include trying to multiply `gasCost` with `gas` without using `bigint`:s.

It's unclear whether any other DSL could be more secure; since there's always the possibility of erroneously implementing a rule.


## Credential management

The ability to auto-approve transaction means that the signer needs to have necessary credentials to decrypt keyfiles. These passwords are hereafter called `ksp` (keystore pass).

### Example implementation

Upon startup of the signer, the signer is given a switch: `--seed <path/to/masterseed>`
The `seed` contains a blob of bytes, which is the master seed for the `signer`.

The `signer` uses the `seed` to:

* Generate the `path` where the settings are stored.
  * `./settings/1df094eb-c2b1-4689-90dd-790046d38025/vault.dat`
  * `./settings/1df094eb-c2b1-4689-90dd-790046d38025/rules.js`
* Generate the encryption password for `vault.dat`.

The `vault.dat` would be an encrypted container storing the following information:

* `ksp` entries
* `sha256` hash of `rules.js`
* Information about pair:ed callers (not yet specified)

### Security considerations

This would leave it up to the user to ensure that the `path/to/masterseed` is handled in a secure way. It's difficult to get around this, although one could
imagine leveraging OS-level keychains where supported. The setup is however in general similar to how ssh-keys are  stored in `.ssh/`.


# Implementation status

This is now implemented (with ephemeral non-encrypted storage for now, so not yet enabled).

## Example 1: ruleset for a rate-limited window


```js
function big(str) {
	if (str.slice(0, 2) == "0x") {
		return new BigNumber(str.slice(2), 16)
	}
	return new BigNumber(str)
}

// Time window: 1 week
var window = 1000* 3600*24*7;

// Limit : 1 ether
var limit = new BigNumber("1e18");

function isLimitOk(transaction) {
	var value = big(transaction.value)
	// Start of our window function
	var windowstart = new Date().getTime() - window;

	var txs = [];
	var stored = storage.get('txs');

	if (stored != "") {
		txs = JSON.parse(stored)
	}
	// First, remove all that have passed out of the time-window
	var newtxs = txs.filter(function(tx){return tx.tstamp > windowstart});
	console.log(txs, newtxs.length);

	// Secondly, aggregate the current sum
	sum = new BigNumber(0)

	sum = newtxs.reduce(function(agg, tx){ return big(tx.value).plus(agg)}, sum);
	console.log("ApproveTx > Sum so far", sum);
	console.log("ApproveTx > Requested", value.toNumber());

	// Would we exceed weekly limit ?
	return sum.plus(value).lt(limit)

}
function ApproveTx(r) {
	if (isLimitOk(r.transaction)) {
		return "Approve"
	}
	return "Nope"
}

/**
* OnApprovedTx(str) is called when a transaction has been approved and signed. The parameter
	* 'response_str' contains the return value that will be sent to the external caller.
* The return value from this method is ignore - the reason for having this callback is to allow the
* ruleset to keep track of approved transactions.
*
* When implementing rate-limited rules, this callback should be used.
* If a rule responds with neither 'Approve' nor 'Reject' - the tx goes to manual processing. If the user
* then accepts the transaction, this method will be called.
*
* TLDR; Use this method to keep track of signed transactions, instead of using the data in ApproveTx.
*/
function OnApprovedTx(resp) {
	var value = big(resp.tx.value)
	var txs = []
	// Load stored transactions
	var stored = storage.get('txs');
	if (stored != "") {
		txs = JSON.parse(stored)
	}
	// Add this to the storage
	txs.push({tstamp: new Date().getTime(), value: value});
	storage.put("txs", JSON.stringify(txs));
}
```

## Example 2: allow destination

```js
function ApproveTx(r) {
	if (r.transaction.from.toLowerCase() == "0x0000000000000000000000000000000000001337") {
		return "Approve"
	}
	if (r.transaction.from.toLowerCase() == "0x000000000000000000000000000000000000dead") {
		return "Reject"
	}
	// Otherwise goes to manual processing
}
```

## Example 3: Allow listing

```js
function ApproveListing() {
	return "Approve"
}
```
