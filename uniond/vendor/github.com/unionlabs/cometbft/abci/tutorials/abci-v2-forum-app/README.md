## Forum Application

The **ABCI 2.0 Forum Application** is a demo application where users can come and post messages in a forum running on a
blockchain powered by [CometBFT](https://github.com/cometbft/cometbft) state machine replication engine.

- **Users**

   - Can post messages (by submitting transactions)
   - Can view all the message history (querying the blockchain)
   - Are banned if they post messages that contain curse words (curse words are tracked with vote extensions)

## ABCI 2.0

**This application demonstrates the use of various [ABCI 2.0](https://docs.cometbft.com/v1.0/spec/abci/) methods such as:**

- PrepareProposal
- ProcessProposal
- FinalizeBlock
- ExtendVote
- VerifyVoteExtension
- Commit
- CheckTx
- Query

To follow this tutorial, please check the [Introduction to ABCI 2.0](../../../docs/tutorials/forum-application/1.abci-intro.md) document.

> Many thanks to the original team for brainstorming and bringing forth this idea. Their original repo can be found [here](https://github.com/interchainio/forum)

