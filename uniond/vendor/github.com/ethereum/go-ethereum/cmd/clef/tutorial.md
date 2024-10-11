## Initializing Clef

First things first, Clef needs to store some data itself. Since that data might be sensitive (passwords, signing rules, accounts), Clef's entire storage is encrypted. To support encrypting data, the first step is to initialize Clef with a random master seed, itself too encrypted with your chosen password:

```text
$ clef init

WARNING!

Clef is an account management tool. It may, like any software, contain bugs.

Please take care to
- backup your keystore files,
- verify that the keystore(s) can be opened with your password.

Clef is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
PURPOSE. See the GNU General Public License for more details.

Enter 'ok' to proceed:
> ok

The master seed of clef will be locked with a password.
Please specify a password. Do not forget this password!
Password:
Repeat password:

A master seed has been generated into /home/martin/.clef/masterseed.json

This is required to be able to store credentials, such as:
* Passwords for keystores (used by rule engine)
* Storage for JavaScript auto-signing rules
* Hash of JavaScript rule-file

You should treat 'masterseed.json' with utmost secrecy and make a backup of it!
* The password is necessary but not enough, you need to back up the master seed too!
* The master seed does not contain your accounts, those need to be backed up separately!
```

*For readability purposes, we'll remove the WARNING printout, user confirmation and the unlocking of the master seed in the rest of this document.*

## Remote interactions

Clef is capable of managing both key-file based accounts as well as hardware wallets. To evaluate clef, we're going to point it to our Rinkeby testnet keystore and specify the Rinkeby chain ID for signing (Clef doesn't have a backing chain, so it doesn't know what network it runs on).

```text
$ clef --keystore ~/.ethereum/rinkeby/keystore --chainid 4

INFO [07-01|11:00:46.385] Starting signer                          chainid=4 keystore=$HOME/.ethereum/rinkeby/keystore light-kdf=false advanced=false
DEBUG[07-01|11:00:46.389] FS scan times                            list=3.521941ms set=9.017µs diff=4.112µs
DEBUG[07-01|11:00:46.391] Ledger support enabled
DEBUG[07-01|11:00:46.391] Trezor support enabled via HID
DEBUG[07-01|11:00:46.391] Trezor support enabled via WebUSB
INFO [07-01|11:00:46.391] Audit logs configured                    file=audit.log
DEBUG[07-01|11:00:46.392] IPC registered                           namespace=account
INFO [07-01|11:00:46.392] IPC endpoint opened                      url=$HOME/.clef/clef.ipc
------- Signer info -------
* intapi_version : 7.0.0
* extapi_version : 6.0.0
* extapi_http : n/a
* extapi_ipc : $HOME/.clef/clef.ipc
```

By default, Clef starts up in CLI (Command Line Interface) mode. Arbitrary remote processes may *request* account interactions (e.g. sign a transaction), which the user will need to individually *confirm*.

To test this out, we can *request* Clef to list all account via its *External API endpoint*:

```text
echo '{"id": 1, "jsonrpc": "2.0", "method": "account_list"}' | nc -U ~/.clef/clef.ipc
```

This will prompt the user within the Clef CLI to confirm or deny the request:

```text
-------- List Account request--------------
A request has been made to list all accounts.
You can select which accounts the caller can see
  [x] 0xD9C9Cd5f6779558b6e0eD4e6Acf6b1947E7fA1F3
    URL: keystore://$HOME/.ethereum/rinkeby/keystore/UTC--2017-04-14T15-15-00.327614556Z--d9c9cd5f6779558b6e0ed4e6acf6b1947e7fa1f3
  [x] 0x086278A6C067775F71d6B2BB1856Db6E28c30418
    URL: keystore://$HOME/.ethereum/rinkeby/keystore/UTC--2018-02-06T22-53-11.211657239Z--086278a6c067775f71d6b2bb1856db6e28c30418
-------------------------------------------
Request context:
	NA -> NA -> NA

Additional HTTP header data, provided by the external caller:
	User-Agent:
	Origin:
Approve? [y/N]:
>
```

Depending on whether we approve or deny the request, the original NetCat process will get:

```text
{"jsonrpc":"2.0","id":1,"result":["0xd9c9cd5f6779558b6e0ed4e6acf6b1947e7fa1f3","0x086278a6c067775f71d6b2bb1856db6e28c30418"]}

or

{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"Request denied"}}
```

Apart from listing accounts, you can also *request* creating a new account; signing transactions and data; and recovering signatures. You can find the available methods in the Clef [External API Spec](https://github.com/ethereum/go-ethereum/tree/master/cmd/clef#external-api-1) and the [External API Changelog](https://github.com/ethereum/go-ethereum/blob/master/cmd/clef/extapi_changelog.md).

*Note, the number of things you can do from the External API is deliberately small, since we want to limit the power of remote calls by as much as possible! Clef has an [Internal API](https://github.com/ethereum/go-ethereum/tree/master/cmd/clef#ui-api-1) too for the UI (User Interface) which is much richer and can support custom interfaces on top. But that's out of scope here.*

## Automatic rules

For most users, manually confirming every transaction is the way to go. However, there are cases when it makes sense to set up some rules which permit Clef to sign a transaction without prompting the user. One such example would be running a signer on Rinkeby or other PoA networks.

For starters, we can create a rule file that automatically permits anyone to list our available accounts without user confirmation. The rule file is a tiny JavaScript snippet that you can program however you want:

```js
function ApproveListing() {
    return "Approve"
}
```

Of course, Clef isn't going to just accept and run arbitrary scripts you give it, that would be dangerous if someone changes your rule file! Instead, you need to explicitly *attest* the rule file, which entails injecting its hash into Clef's secure store.

```text
$ sha256sum rules.js
645b58e4f945e24d0221714ff29f6aa8e860382ced43490529db1695f5fcc71c  rules.js

$ clef attest 645b58e4f945e24d0221714ff29f6aa8e860382ced43490529db1695f5fcc71c
Decrypt master seed of clef
Password:
INFO [07-01|13:25:03.290] Ruleset attestation updated              sha256=645b58e4f945e24d0221714ff29f6aa8e860382ced43490529db1695f5fcc71c
```

At this point, we can start Clef with the rule file:

```text
$ clef --keystore ~/.ethereum/rinkeby/keystore --chainid 4 --rules rules.js

INFO [07-01|13:39:49.726] Rule engine configured                   file=rules.js
INFO [07-01|13:39:49.726] Starting signer                          chainid=4 keystore=$HOME/.ethereum/rinkeby/keystore light-kdf=false advanced=false
DEBUG[07-01|13:39:49.726] FS scan times                            list=35.15µs set=4.251µs diff=2.766µs
DEBUG[07-01|13:39:49.727] Ledger support enabled
DEBUG[07-01|13:39:49.727] Trezor support enabled via HID
DEBUG[07-01|13:39:49.727] Trezor support enabled via WebUSB
INFO [07-01|13:39:49.728] Audit logs configured                    file=audit.log
DEBUG[07-01|13:39:49.728] IPC registered                           namespace=account
INFO [07-01|13:39:49.728] IPC endpoint opened                      url=$HOME/.clef/clef.ipc
------- Signer info -------
* intapi_version : 7.0.0
* extapi_version : 6.0.0
* extapi_http : n/a
* extapi_ipc : $HOME/.clef/clef.ipc
```

Any account listing *request* will now be auto-approved by the rule file:

```text
$ echo '{"id": 1, "jsonrpc": "2.0", "method": "account_list"}' | nc -U ~/.clef/clef.ipc
{"jsonrpc":"2.0","id":1,"result":["0xd9c9cd5f6779558b6e0ed4e6acf6b1947e7fa1f3","0x086278a6c067775f71d6b2bb1856db6e28c30418"]}
```

## Under the hood

While doing the operations above, these files have been created:

```text
$ ls -laR ~/.clef/

$HOME/.clef/:
total 24
drwxr-x--x   3 user user  4096 Jul  1 13:45 .
drwxr-xr-x 102 user user 12288 Jul  1 13:39 ..
drwx------   2 user user  4096 Jul  1 13:25 02f90c0603f4f2f60188
-r--------   1 user user   868 Jun 28 13:55 masterseed.json

$HOME/.clef/02f90c0603f4f2f60188:
total 12
drwx------ 2 user user 4096 Jul  1 13:25 .
drwxr-x--x 3 user user 4096 Jul  1 13:45 ..
-rw------- 1 user user  159 Jul  1 13:25 config.json

$ cat ~/.clef/02f90c0603f4f2f60188/config.json
{"ruleset_sha256":{"iv":"SWWEtnl+R+I+wfG7","c":"I3fjmwmamxVcfGax7D0MdUOL29/rBWcs73WBILmYK0o1CrX7wSMc3y37KsmtlZUAjp0oItYq01Ow8VGUOzilG91tDHInB5YHNtm/YkufEbo="}}
```

In `$HOME/.clef`, the `masterseed.json` file was created, containing the master seed. This seed was then used to derive a few other things:

- **Vault location**: in this case `02f90c0603f4f2f60188`.
   - If you use a different master seed, a different vault location will be used that does not conflict with each other (e.g. `clef --signersecret /path/to/file`). This allows you to run multiple instances of Clef, each with its own rules (e.g. mainnet + testnet).
- **`config.json`**: the encrypted key/value storage for configuration data, currently only containing the key `ruleset_sha256`, the attested hash of the automatic rules to use.

## Advanced rules

In order to make more useful rules - like signing transactions - the signer needs access to the passwords needed to unlock keys from the keystore. You can inject an unlock password via `clef setpw`.

```text
$ clef setpw 0xd9c9cd5f6779558b6e0ed4e6acf6b1947e7fa1f3

Please enter a password to store for this address:
Password:
Repeat password:

Decrypt master seed of clef
Password:
INFO [07-01|14:05:56.031] Credential store updated                 key=0xd9c9cd5f6779558b6e0ed4e6acf6b1947e7fa1f3
```

Now let's update the rules to make use of the new credentials:

```js
function ApproveListing() {
    return "Approve"
}

function ApproveSignData(req) {
    if (req.address.toLowerCase() == "0xd9c9cd5f6779558b6e0ed4e6acf6b1947e7fa1f3") {
        if (req.messages[0].value.indexOf("bazonk") >= 0) {
            return "Approve"
        }
        return "Reject"
    }
    // Otherwise goes to manual processing
}
```

In this example:

- Any requests to sign data with the account `0xd9c9...` will be:
    - Auto-approved if the message contains `bazonk`,
    - Auto-rejected if the message does not contain `bazonk`,
- Any other requests will be passed along for manual confirmation.

*Note, to make this example work, please use you own accounts. You can create a new account either via Clef or the traditional account CLI tools. If the latter was chosen, make sure both Clef and Geth use the same keystore by specifying `--keystore path/to/your/keystore` when running Clef.*

Attest the new rule file so that Clef will accept loading it:

```text
$ sha256sum rules.js
f163a1738b649259bb9b369c593fdc4c6b6f86cc87e343c3ba58faee03c2a178  rules.js

$ clef attest f163a1738b649259bb9b369c593fdc4c6b6f86cc87e343c3ba58faee03c2a178
Decrypt master seed of clef
Password:
INFO [07-01|14:11:28.509] Ruleset attestation updated              sha256=f163a1738b649259bb9b369c593fdc4c6b6f86cc87e343c3ba58faee03c2a178
```

Restart Clef with the new rules in place:

```
$ clef --keystore ~/.ethereum/rinkeby/keystore --chainid 4 --rules rules.js

INFO [07-01|14:12:41.636] Rule engine configured                   file=rules.js
INFO [07-01|14:12:41.636] Starting signer                          chainid=4 keystore=$HOME/.ethereum/rinkeby/keystore light-kdf=false advanced=false
DEBUG[07-01|14:12:41.636] FS scan times                            list=46.722µs set=4.47µs diff=2.157µs
DEBUG[07-01|14:12:41.637] Ledger support enabled
DEBUG[07-01|14:12:41.637] Trezor support enabled via HID
DEBUG[07-01|14:12:41.638] Trezor support enabled via WebUSB
INFO [07-01|14:12:41.638] Audit logs configured                    file=audit.log
DEBUG[07-01|14:12:41.638] IPC registered                           namespace=account
INFO [07-01|14:12:41.638] IPC endpoint opened                      url=$HOME/.clef/clef.ipc
------- Signer info -------
* intapi_version : 7.0.0
* extapi_version : 6.0.0
* extapi_http : n/a
* extapi_ipc : $HOME/.clef/clef.ipc
```

Then test signing, once with `bazonk` and once without:

```
$ echo '{"id": 1, "jsonrpc":"2.0", "method":"account_signData", "params":["data/plain", "0xd9c9cd5f6779558b6e0ed4e6acf6b1947e7fa1f3", "0x202062617a6f6e6b2062617a2067617a0a"]}' | nc -U ~/.clef/clef.ipc
{"jsonrpc":"2.0","id":1,"result":"0x4f93e3457027f6be99b06b3392d0ebc60615ba448bb7544687ef1248dea4f5317f789002df783979c417d969836b6fda3710f5bffb296b4d51c8aaae6e2ac4831c"}

$ echo '{"id": 1, "jsonrpc":"2.0", "method":"account_signData", "params":["data/plain", "0xd9c9cd5f6779558b6e0ed4e6acf6b1947e7fa1f3", "0x2020626f6e6b2062617a2067617a0a"]}' | nc -U ~/.clef/clef.ipc
{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"Request denied"}}
```

Meanwhile, in the Clef output log you can see:
```text
INFO [02-21|14:42:41] Op approved
INFO [02-21|14:42:56] Op rejected
```

The signer also stores all traffic over the external API in a log file. The last 4 lines shows the two requests and their responses:

```text
$ tail -n 4 audit.log
t=2019-07-01T15:52:14+0300 lvl=info msg=SignData   api=signer type=request  metadata="{\"remote\":\"NA\",\"local\":\"NA\",\"scheme\":\"NA\",\"User-Agent\":\"\",\"Origin\":\"\"}" addr="0xd9c9cd5f6779558b6e0ed4e6acf6b1947e7fa1f3 [chksum INVALID]" data=0x202062617a6f6e6b2062617a2067617a0a content-type=data/plain
t=2019-07-01T15:52:14+0300 lvl=info msg=SignData   api=signer type=response data=4f93e3457027f6be99b06b3392d0ebc60615ba448bb7544687ef1248dea4f5317f789002df783979c417d969836b6fda3710f5bffb296b4d51c8aaae6e2ac4831c error=nil
t=2019-07-01T15:52:23+0300 lvl=info msg=SignData   api=signer type=request  metadata="{\"remote\":\"NA\",\"local\":\"NA\",\"scheme\":\"NA\",\"User-Agent\":\"\",\"Origin\":\"\"}" addr="0xd9c9cd5f6779558b6e0ed4e6acf6b1947e7fa1f3 [chksum INVALID]" data=0x2020626f6e6b2062617a2067617a0a     content-type=data/plain
t=2019-07-01T15:52:23+0300 lvl=info msg=SignData   api=signer type=response data=                                     error="Request denied"
```

For more details on writing automatic rules, please see the [rules spec](https://github.com/ethereum/go-ethereum/blob/master/cmd/clef/rules.md).

## Geth integration

Of course, as awesome as Clef is, it's not feasible to interact with it via JSON RPC by hand. Long term, we're hoping to convince the general Ethereum community to support Clef as a general signer (it's only 3-5 methods), thus allowing your favorite DApp, Metamask, MyCrypto, etc to request signatures directly.

Until then however, we're trying to pave the way via Geth. Geth v1.9.0 has built in support via `--signer <API endpoint>` for using a local or remote Clef instance as an account backend!

We can try this by running Clef with our previous rules on Rinkeby (for now it's a good idea to allow auto-listing accounts, since Geth likes to retrieve them once in a while).

```text
$ clef --keystore ~/.ethereum/rinkeby/keystore --chainid 4 --rules rules.js
```

In a different window we can start Geth, list our accounts, even list our wallets to see where the accounts originate from:

```text
$ geth --rinkeby --signer=~/.clef/clef.ipc console

> eth.accounts
["0xd9c9cd5f6779558b6e0ed4e6acf6b1947e7fa1f3", "0x086278a6c067775f71d6b2bb1856db6e28c30418"]

> personal.listWallets
[{
    accounts: [{
        address: "0xd9c9cd5f6779558b6e0ed4e6acf6b1947e7fa1f3",
        url: "extapi://$HOME/.clef/clef.ipc"
    }, {
        address: "0x086278a6c067775f71d6b2bb1856db6e28c30418",
        url: "extapi://$HOME/.clef/clef.ipc"
    }],
    status: "ok [version=6.0.0]",
    url: "extapi://$HOME/.clef/clef.ipc"
}]

> eth.sendTransaction({from: eth.accounts[0], to: eth.accounts[0]})
```

Lastly, when we requested a transaction to be sent, Clef prompted us in the original window to approve it:

```text
--------- Transaction request-------------
to:       0xD9C9Cd5f6779558b6e0eD4e6Acf6b1947E7fA1F3
from:     0xD9C9Cd5f6779558b6e0eD4e6Acf6b1947E7fA1F3 [chksum ok]
value:    0 wei
gas:      0x5208 (21000)
gasprice: 1000000000 wei
nonce:    0x2366 (9062)

Request context:
	NA -> NA -> NA

Additional HTTP header data, provided by the external caller:
	User-Agent:
	Origin:
-------------------------------------------
Approve? [y/N]:
> y
```

:boom:

*Note, if you enable the external signer backend in Geth, all other account management is disabled. This is because long term we want to remove account management from Geth.*
