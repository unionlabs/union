---
order: 1
parent:
  title: ABCI 2.0
  order: 3
---

# ABCI 2.0

## Introduction

ABCI 2.0 is a major evolution of ABCI (**A**pplication **B**lock**c**hain **I**nterface).
ABCI is the interface between CometBFT (a state-machine
replication engine) and the actual state machine being replicated (i.e., the Application).
The API consists of a set of _methods_, each with a corresponding `Request` and `Response`
message type.

> Note: ABCI 2.0 is colloquially called ABCI++. To be precise in these documents, we will refer to the exact version of ABCI under discussion, currently 2.0.

The methods are always initiated by CometBFT. The Application implements its logic
for handling all ABCI methods.
Thus, CometBFT always sends the `*Request` messages and receives the `*Response` messages
in return.

All ABCI messages and methods are defined in [protocol buffers](https://github.com/cometbft/cometbft/blob/main/proto/cometbft/abci/v1/types.proto).
This allows CometBFT to run with applications written in many programming languages.

This specification is split as follows:

- [Overview and basic concepts](./abci++_basic_concepts.md) - interface's overview and concepts
  needed to understand other parts of this specification.
- [Methods](./abci++_methods.md) - complete details on all ABCI methods
  and message types.
- [Requirements for the Application](./abci++_app_requirements.md) - formal requirements
  on the Application's logic to ensure CometBFT properties such as liveness. These requirements define what
  CometBFT expects from the Application; second part on managing ABCI application state and related topics.
- [CometBFT's expected behavior](./abci++_comet_expected_behavior.md) - specification of
  how the different ABCI methods may be called by CometBFT. This explains what the Application
  is to expect from CometBFT.
- [Example scenarios](./abci++_example_scenarios.md) - specific scenarios showing why the Application needs to account
for any CometBFT's behaviour prescribed by the specification.
- [Client and Server](./abci++_client_server.md) - for those looking to implement their
  own ABCI application servers.
