# Ivy Proofs

```copyright
Copyright (c) 2020 Galois, Inc.
SPDX-License-Identifier: Apache-2.0
```

## Contents

This folder contains:

* `tendermint.ivy`, a specification of Tendermint algorithm as described in *The latest gossip on BFT consensus* by E. Buchman, J. Kwon, Z. Milosevic.
* `abstract_tendermint.ivy`, a more abstract specification of Tendermint that is more verification-friendly.
* `classic_safety.ivy`, a proof that Tendermint satisfies the classic safety property of BFT consensus: if every two quorums have a well-behaved node in common, then no two well-behaved nodes ever disagree.
* `accountable_safety_1.ivy`, a proof that, assuming every quorum contains at least one well-behaved node, if two well-behaved nodes disagree, then there is evidence demonstrating at least f+1 nodes misbehaved.
* `accountable_safety_2.ivy`, a proof that, regardless of any assumption about quorums, well-behaved nodes cannot be framed by malicious nodes. In other words, malicious nodes can never construct evidence that incriminates a well-behaved node.
* `network_shim.ivy`, the network model and a convenience `shim` object to interface with the Tendermint specification.
* `domain_model.ivy`, a specification of the domain model underlying the Tendermint specification, i.e. rounds, value, quorums, etc.

All specifications and proofs are written in [Ivy](https://github.com/kenmcmil/ivy).

The license above applies to all files in this folder.


## Building and running

The easiest way to check the proofs is to use [Docker](https://www.docker.com/).

1. Install [Docker](https://docs.docker.com/get-docker/) and [Docker Compose](https://docs.docker.com/compose/install/).
2. Build a Docker image: `docker-compose build`
3. Run the proofs inside the Docker container: `docker-compose run
tendermint-proof`. This will check all the proofs with the `ivy_check`
command and write the output of `ivy_check` to a subdirectory of `./output/'
