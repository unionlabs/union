# Security

> **IMPORTANT**: If you find a security issue, you can
report it to our [bug bounty program](https://hackerone.com/cosmos) on HackerOne. *DO NOT* open a public issue on the repository.

## Bug Bounty

As part of our [Coordinated Vulnerability Disclosure Policy](https://tendermint.com/security), we operate a
[bug bounty program](https://hackerone.com/cosmos) with Hacker One.

See the policy linked above for more details on submissions and rewards and read
this [blog post](https://blog.cosmos.network/bug-bounty-program-for-tendermint-cosmos-833c67693586) for the program scope.

The following is a list of examples of the kinds of bugs we're most interested
in for the Cosmos SDK. See [here](https://github.com/tendermint/tendermint/blob/master/SECURITY.md) for vulnerabilities we are interested
in for Tendermint and other lower-level libraries (eg. [IAVL](https://github.com/cosmos/iavl)).

### Core packages

* [`/baseapp`](https://github.com/cosmos/cosmos-sdk/tree/main/baseapp)
* [`/crypto`](https://github.com/cosmos/cosmos-sdk/tree/main/crypto)
* [`/types`](https://github.com/cosmos/cosmos-sdk/tree/main/types)
* [`/store`](https://github.com/cosmos/cosmos-sdk/tree/main/store)

### Modules

* [`x/auth`](https://github.com/cosmos/cosmos-sdk/tree/main/x/auth)
* [`x/bank`](https://github.com/cosmos/cosmos-sdk/tree/main/x/bank)
* [`x/capability`](https://github.com/cosmos/cosmos-sdk/tree/main/x/capability)
* [`x/staking`](https://github.com/cosmos/cosmos-sdk/tree/main/x/staking)
* [`x/slashing`](https://github.com/cosmos/cosmos-sdk/tree/main/x/slashing)
* [`x/evidence`](https://github.com/cosmos/cosmos-sdk/tree/main/x/evidence)
* [`x/distribution`](https://github.com/cosmos/cosmos-sdk/tree/main/x/distribution)
* [`x/mint`](https://github.com/cosmos/cosmos-sdk/tree/main/x/mint)

We are interested in bugs in other modules, however the above are most likely to
have significant vulnerabilities, due to the complexity / nuance involved. We
also recommend you to read the [specification](https://github.com/cosmos/cosmos-sdk/blob/main/docs/building-modules/README.md) of each module before digging into
the code.

### How we process Tx parameters

* Integer operations on tx parameters, especially `sdk.Int` / `sdk.Dec`
* Gas calculation & parameter choices
* Tx signature verification (see [`x/auth/ante`](https://github.com/cosmos/cosmos-sdk/tree/main/x/auth/ante))
* Possible Node DoS vectors (perhaps due to gas weighting / non constant timing)

### Handling private keys

* HD key derivation, local and Ledger, and all key-management functionality
* Side-channel attack vectors with our implementations
    * e.g. key exfiltration based on time or memory-access patterns when decrypting privkey

## Disclosure Process

The Cosmos SDK team uses the following disclosure process:

1. After a security report is received, the Cosmos SDK team works to verify the issue and confirm its severity level using Common Vulnerability Scoring System (CVSS).
1. The Cosmos SDK team collaborates with the CometBFT and Gaia teams to determine the vulnerability’s potential impact on the Cosmos Hub and partners.
1. Patches are prepared in private repositories for eligible releases of Cosmos SDK. See [Stable Release Policy](https://github.com/cosmos/cosmos-sdk/blob/main/RELEASE_PROCESS.md#stable-release-policy) for a list of eligible releases.
1. If it is determined that a CVE-ID is required, we request a CVE through a CVE Numbering Authority.
1. We notify the community that a security release is coming to give users time to prepare their systems for the update. Notifications can include forum posts, tweets, and emails to partners and validators.
1. 24 hours after the notification, fixes are applied publicly and new releases are issued.
1. The Gaia team updates their CometBFT and Cosmos SDK dependencies to use these releases and then issues new Gaia releases.
1. After releases are available for CometBFT, Cosmos SDK, and Gaia, we notify the community again through the same channels. We also publish a Security Advisory on Github and publish the CVE, as long as the Security Advisory and the CVE do not include information on how to exploit these vulnerabilities beyond the information that is available in the patch.
1. After the community is notified, CometBFT pays out any relevant bug bounties to submitters.
1. One week after the releases go out, we publish a post with details and our response to the vulnerability.

This process can take some time. Every effort is made to handle the bug in as timely a manner as possible. However, it's important that we follow this security process to ensure that disclosures are handled consistently and to keep Cosmos SDK and its downstream dependent projects--including but not limited to Gaia and the Cosmos Hub--as secure as possible.

### Disclosure Communications

Communications to partners usually include the following details:

1. Affected version or versions
1. New release version
1. Impact on user funds
1. For timed releases, a date and time that the new release will be made available
1. Impact on the partners if upgrades are not completed in a timely manner
1. Potential required actions if an adverse condition arises during the security release process

An example notice looks like:

```text
Dear Cosmos SDK partners,

A critical security vulnerability has been identified in Cosmos SDK vX.X.X.
User funds are NOT at risk; however, the vulnerability can result in a chain halt.

This notice is to inform you that on [[**March 1 at 1pm EST/6pm UTC**]], we will be releasing Cosmos SDK vX.X.Y to fix the security issue.
We ask all validators to upgrade their nodes ASAP.

If the chain halts, validators with sufficient voting power must upgrade and come online for the chain to resume.
```

### Example Timeline

The following timeline is an example of triage and response. Each task identifies the required roles and team members; however, multiple people can play each role and each person may play multiple roles.

#### 24+ Hours Before Release Time

1. Request CVE number (ADMIN)
1. Gather emails and other contact info for validators (COMMS LEAD)
1. Test fixes on a testnet  (COSMOS SDK ENG)
1. Write “Security Advisory” for forum (COSMOS SDK LEAD)

#### 24 Hours Before Release Time

1. Post “Security Advisory” pre-notification on forum (COSMOS SDK LEAD)
1. Post Tweet linking to forum post (COMMS LEAD)
1. Announce security advisory/link to post in various other social channels (Telegram, Discord) (COMMS LEAD)
1. Send emails to partners or other users (PARTNERSHIPS LEAD)

#### Release Time

1. Cut Cosmos SDK releases for eligible versions (COSMOS SDK ENG)
1. Cut Gaia release for eligible versions (GAIA ENG)
1. Post “Security releases” on forum (COSMOS SDK LEAD)
1. Post new Tweet linking to forum post (COMMS LEAD)
1. Remind everyone using social channels (Telegram, Discord)  that the release is out (COMMS LEAD)
1. Send emails to validators and other users (COMMS LEAD)
1. Publish Security Advisory and CVE if the CVE has no sensitive information (ADMIN)

#### After Release Time

1. Write forum post with exploit details (COSMOS SDK LEAD)
1. Approve payout on HackerOne for submitter (ADMIN)

#### 7 Days After Release Time

1. Publish CVE if it has not yet been published (ADMIN)
1. Publish forum post with exploit details (COSMOS SDK ENG, COSMOS SDK LEAD)
