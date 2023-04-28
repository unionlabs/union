---
title: 'Mainnet Overview'
---

# Mainnet

The union.fi mainnet release date will be announced after the public testnet period has ended. 

## Genesis 

The network will be bootstrapped with the top validators from the public testnet

## Architecture

### Topology

The network has an initial active validator set size of 64, which may be increased later through governance proposals. Since the security of Union relies on the amount of tokens staked, not the total number of validators, it was chosen to optimize for proving times and peer-to-peer network performance.

<img src="/img/mainnet-overview.drawio.svg"  width="100%" height="30%"/>

### Prover

The prover connects over the RPC interface to construct CometBLS-Groth16 proofs. It submits these to the Ethereum.