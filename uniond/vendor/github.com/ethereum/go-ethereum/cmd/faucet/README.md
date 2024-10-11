# Faucet

The `faucet` is a simplistic web application with the goal of distributing small amounts of Ether in private and test networks.

Users need to post their Ethereum addresses to fund in a Twitter status update or public Facebook post and share the link to the faucet. The faucet will in turn deduplicate user requests and send the Ether. After a funding round, the faucet prevents the same user from requesting again for a pre-configured amount of time, proportional to the amount of Ether requested.

## Operation

The `faucet` is a single binary app (everything included) with all configurations set via command line flags and a few files.

First things first, the `faucet` needs to connect to an Ethereum network, for which it needs the necessary genesis and network infos. Each of the following flags must be set:

- `-genesis` is a path to a file containing the network `genesis.json`. or using:
  - `-goerli` with the faucet with Görli network config
  - `-sepolia` with the faucet with Sepolia network config
- `-network` is the devp2p network id used during connection
- `-bootnodes` is a list of `enode://` ids to join the network through

The `faucet` will use the `les` protocol to join the configured Ethereum network and will store its data in `$HOME/.faucet` (currently not configurable).

## Funding

To be able to distribute funds, the `faucet` needs access to an already funded Ethereum account. This can be configured via:

- `-account.json` is a path to the Ethereum account's JSON key file
- `-account.pass` is a path to a text file with the decryption passphrase

The faucet is able to distribute various amounts of Ether in exchange for various timeouts. These can be configured via:

- `-faucet.amount` is the number of Ethers to send by default
- `-faucet.minutes` is the time to wait before allowing a rerequest
- `-faucet.tiers` is the funding tiers to support  (x3 time, x2.5 funds)

## Sybil protection

To prevent the same user from exhausting funds in a loop, the `faucet` ties requests to social networks and captcha resolvers.

Captcha protection uses Google's invisible ReCaptcha, thus the `faucet` needs to run on a live domain. The domain needs to be registered in Google's systems to retrieve the captcha API token and secrets. After doing so, captcha protection may be enabled via:

- `-captcha.token` is the API token for ReCaptcha
- `-captcha.secret` is the API secret for ReCaptcha

Sybil protection via Twitter requires an API key as of 15th December, 2020. To obtain it, a Twitter user must be upgraded to developer status and a new Twitter App deployed with it. The app's `Bearer` token is required by the faucet to retrieve tweet data:

- `-twitter.token` is the Bearer token for `v2` API access
- `-twitter.token.v1` is the Bearer token for `v1` API access

Sybil protection via Facebook uses the website to directly download post data thus does not currently require an API configuration. 

## Miscellaneous

Beside the above - mostly essential - CLI flags, there are a number that can be used to fine-tune the `faucet`'s operation. Please see `faucet --help` for a full list.
