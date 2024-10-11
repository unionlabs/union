# Upgrading

With stargate, we have access to the `x/upgrade` module, which we can use to perform
inline upgrades. Please first read both the basic 
[x/upgrade spec](https://github.com/cosmos/cosmos-sdk/blob/master/x/upgrade/spec/01_concepts.md)
and [go docs](https://godoc.org/github.com/cosmos/cosmos-sdk/x/upgrade#hdr-Performing_Upgrades)
for the background on the module.

In this case, we will demo an update with no state migration. This is for cases when
there is a state-machine-breaking (but not state-breaking) bugfix or enhancement.
There are some
[open issues running some state migrations](https://github.com/cosmos/cosmos-sdk/issues/8265)
and we will wait for that to be fixed before trying those.

The following will lead through running an upgrade on a local node, but the same process
would work on a real network (with more ops and governance coordination).

## Setup

We need to have two different versions of `wasmd` which depend on state-compatible
versions of the Cosmos SDK. We only focus on upgrade starting with stargate. You will
have to use the "dump state and restart" approach to move from launchpad to stargate.

For this demo, we will show an upgrade from `v0.14.0` to musselnet branch.

### Handler

You will need to register a handler for the upgrade. This is specific to a particular
testnet and upgrade path, and the default `wasmd` will never have a registered handler
on master. In this case, we make a `musselnet` branch off of `v0.14.0` just
registering one handler with a given name. 

Look at [PR 351](https://github.com/CosmWasm/wasmd/pull/351/files) for an example
of a minimal handler. We do not make any state migrations, but rather use this
as a flag to coordinate all validators to stop the old version at one height, and
start the specified v2 version on the next block.

### Prepare binaries

Let's get the two binaries we want to test, the pre-upgrade and the post-upgrade
binaries. In this case the pre-release is already a published to docker hub and
can be downloaded simply via:

`docker pull cosmwasm/wasmd:v0.14.0`

(If this is not yet released, build it from the tip of master)

The post-release is not published, so we can build it ourselves. Check out this
`wasmd` repo, and the proper `musselnet` branch:

```
# use musselnet-v2 tag once that exists
git checkout musselnet
docker build . -t wasmd:musselnet-v2
```

Verify they are both working for you locally:

```
docker run cosmwasm/wasmd:v0.14.0 wasmd version
docker run wasmd:musselnet-v2 wasmd version
```

## Start the pre-release chain

Follow the normal setup stage, but in this case we will want to have super short
governance voting period, 5 minutes rather than 2 days (or 2 weeks!).

**Setup a client with private key**

```sh
## TODO: I think we need to do this locally???
docker volume rm -f musselnet_client

docker run --rm -it \
    -e PASSWORD=1234567890 \
    --mount type=volume,source=musselnet_client,target=/root \
    cosmwasm/wasmd:v0.14.0 /opt/setup_wasmd.sh

# enter "1234567890" when prompted
docker run --rm -it \
    --mount type=volume,source=musselnet_client,target=/root \
    cosmwasm/wasmd:v0.14.0 wasmd keys show -a validator
# use the address returned above here
CLIENT=wasm1anavj4eyxkdljp27sedrdlt9dm26c8a7a8p44l
```

**Setup the blockchain node**

```sh
docker volume rm -f musselnet

# add your testing address here, so you can do something with the client
docker run --rm -it \
    --mount type=volume,source=musselnet,target=/root \
    cosmwasm/wasmd:v0.14.0 /opt/setup_wasmd.sh $CLIENT

# Update the voting times in the genesis file
docker run --rm -it \
    --mount type=volume,source=musselnet,target=/root \
    cosmwasm/wasmd:v0.14.0 sed -ie 's/172800s/300s/' /root/.wasmd/config/genesis.json

# start up the blockchain and all embedded servers as one process
docker run --rm -it -p 26657:26657 -p 26656:26656 -p 1317:1317 \
    --mount type=volume,source=musselnet,target=/root \
    cosmwasm/wasmd:v0.14.0 /opt/run_wasmd.sh
```

## Sanity checks

Let's use our client node to query the current state and send some tokens to a
random address:

```sh
RCPT=wasm1pypadqklna33nv3gl063rd8z9q8nvauaalz820

# note --network=host so it can connect to the other docker image
docker run --rm -it \
    --mount type=volume,source=musselnet_client,target=/root \
    --network=host \
    cosmwasm/wasmd:v0.14.0 wasmd \
    query bank balances $CLIENT

docker run --rm -it \
    --mount type=volume,source=musselnet_client,target=/root \
    --network=host \
    cosmwasm/wasmd:v0.14.0 wasmd \
    query bank balances $RCPT

docker run --rm -it \
    --mount type=volume,source=musselnet_client,target=/root \
    --network=host \
    cosmwasm/wasmd:v0.14.0 wasmd \
    tx send validator $RCPT 500000ucosm,600000ustake --chain-id testing

docker run --rm -it \
    --mount type=volume,source=musselnet_client,target=/root \
    --network=host \
    cosmwasm/wasmd:v0.14.0 wasmd \
    query bank balances $RCPT
```

## Take majority control of the chain

In genesis we have a valiator with 250 million `ustake` bonded. We want to be easily
able to pass a proposal with our client. Let us bond 700 million `ustake` to ensure
we have > 67% of the voting power and will pass with the validator not voting.

```sh
# get the "operator_address" (wasmvaloper...) from here
docker run --rm -it \
    --mount type=volume,source=musselnet_client,target=/root \
    --network=host \
    cosmwasm/wasmd:v0.14.0 wasmd \
    query staking validators
VALIDATOR=......

# and stake here
docker run --rm -it \
    --mount type=volume,source=musselnet_client,target=/root \
    --network=host \
    cosmwasm/wasmd:v0.14.0 wasmd \
    tx staking delegate $VALIDATOR 750000000ustake \
    --from validator --chain-id testing
```

## Vote on the upgrade

Now that we see the chain is running and producing blocks, and our client has
enough token to control the netwrok, let's create a governance
upgrade proposal for the new chain to move to `musselnet-v2` (this must be the
same name as we use in the handler we created above, change this to match what
you put in your handler):

```sh
# create the proposal
# check the current height and add 100-200 or so for the upgrade time
# (the voting period is ~60 blocks)
docker run --rm -it \
    --mount type=volume,source=musselnet_client,target=/root \
    --network=host \
    cosmwasm/wasmd:v0.14.0 wasmd \
    tx gov submit-proposal software-upgrade musselnet-v2 \
    --upgrade-height=500 --deposit=10000000ustake \
    --title="Upgrade" --description="Upgrade to musselnet-v2" \
    --from validator --chain-id testing

# make sure it looks good
docker run --rm -it \
    --mount type=volume,source=musselnet_client,target=/root \
    --network=host \
    cosmwasm/wasmd:v0.14.0 wasmd \
    query gov proposal 1

# vote for it
docker run --rm -it \
    --mount type=volume,source=musselnet_client,target=/root \
    --network=host \
    cosmwasm/wasmd:v0.14.0 wasmd \
    tx gov vote 1 yes \
    --from validator --chain-id testing

# ensure vote was counted
docker run --rm -it \
    --mount type=volume,source=musselnet_client,target=/root \
    --network=host \
    cosmwasm/wasmd:v0.14.0 wasmd \
    query gov votes 1
```

## Vote on the upgrade (Starting from wasmd v0.40.0)

Starting from `v0.40.0` of `wasmd`, which incorporates cosmos-sdk `v0.47.x`,
there have been changes in how upgrade proposals are handled. Below, 
we provide an example of how to achieve the same outcome as described 
in the preceding section.

Please be aware that some commands have been replaced by an interactive 
Command Line Interface (CLI), and the process of submitting a proposal 
is now divided into two distinct steps: `proposal creation` and `proposal submission`.

```sh
# create the proposal
docker run --rm -it \
    --mount type=volume,source=musselnet_client,target=/root \
    --network=host \
    cosmwasm/wasmd:v0.40.0 wasmd \
    tx gov draft-proposal \
    --from validator --chain-id testing

# choose <software-upgrade> from the interactive CLI and fill all the fields
# of the generated json file draft_proposal.json
# example:
{
 "messages": [
  {
   "@type": "/cosmos.upgrade.v1beta1.MsgSoftwareUpgrade",
   "authority": "wasm10d07y265gmmuvt4z0w9aw880jnsr700js7zslc",
   "plan": {
    "name": "Upgrade",
    "time": "0001-01-01T00:00:00Z",
    "height": "500",
    "info": "",
    "upgraded_client_state": null
   }
  }
 ],
 "metadata": "ipfs://CID",
 "deposit": "100000ustake",
 "title": "Upgrade",
 "summary": "summary"
}

# submit the proposal
docker run --rm -it \
    --mount type=volume,source=musselnet_client,target=/root \
    --network=host \
    cosmwasm/wasmd:v0.40.0 wasmd \
    tx gov submit-proposal draft_proposal.json \
    --from validator --chain-id testing

# make sure it looks good
docker run --rm -it \
    --mount type=volume,source=musselnet_client,target=/root \
    --network=host \
    cosmwasm/wasmd:v0.40.0 wasmd \
    query gov proposal 1

# vote for it
docker run --rm -it \
    --mount type=volume,source=musselnet_client,target=/root \
    --network=host \
    cosmwasm/wasmd:v0.40.0 wasmd \
    tx gov vote 1 yes \
    --from validator --chain-id testing

# ensure vote was counted
docker run --rm -it \
    --mount type=volume,source=musselnet_client,target=/root \
    --network=host \
    cosmwasm/wasmd:v0.40.0 wasmd \
    query gov votes 1
```

## Swap out binaries

Now, we just wait about 5 minutes for the vote to pass, and ensure it is passed:

```sh
# make sure it looks good
docker run --rm -it \
    --mount type=volume,source=musselnet_client,target=/root \
    --network=host \
    cosmwasm/wasmd:v0.14.0 wasmd \
    query gov proposal 1
```

After this, we just let the chain run and open the terminal so you can see the log files.
It should keep producing blocks until it hits height 500 (or whatever you set there),
when the process will print a huge stacktrace and hang. Immediately before the stack trace, you
should see a line like this (burried under tons of tendermint logs):

`8:50PM ERR UPGRADE "musselnet-v2" NEEDED at height: 100:`

Kill it with Ctrl-C, and then try to restart with the pre-upgrade version and it should
immediately fail on startup, with the same error message as above.

```sh
docker run --rm -it -p 26657:26657 -p 26656:26656 -p 1317:1317 \
    --mount type=volume,source=musselnet,target=/root \
    cosmwasm/wasmd:v0.14.0 /opt/run_wasmd.sh
```

Then, we start with the post-upgrade version and see it properly update:

```sh
docker run --rm -it -p 26657:26657 -p 26656:26656 -p 1317:1317 \
    --mount type=volume,source=musselnet,target=/root \
    wasmd:musselnet-v2 /opt/run_wasmd.sh
```

On a real network, operators will have to be awake when the upgrade plan is activated
and manually perform this switch, or use some automated tooling like 
[cosmosvisor](https://github.com/cosmos/cosmos-sdk/blob/master/cosmovisor/README.md).

## Check final state

Now that we have upgraded, we can use the new client version. Let's do a brief
sanity check to ensure our balances are proper, and our stake remains
delegated. That and continued block production should be a good sign the upgrade
was successful:

```sh
docker run --rm -it \
    --mount type=volume,source=musselnet_client,target=/root \
    --network=host \
    wasmd:musselnet-v2 wasmd \
    query bank balances $CLIENT

docker run --rm -it \
    --mount type=volume,source=musselnet_client,target=/root \
    --network=host \
    wasmd:musselnet-v2 wasmd \
    query bank balances $RCPT

docker run --rm -it \
    --mount type=volume,source=musselnet_client,target=/root \
    --network=host \
    wasmd:musselnet-v2 wasmd \
    query staking delegations $CLIENT
```
