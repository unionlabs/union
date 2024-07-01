# Voyager Architecture

Relaying is hard. There are several key properties to a reliable relayer, the most important of
which being:

- **Speed.** IBC Relaying is a race to the bottom, with many relayers fighting to be the first to
submit a packet, often times submitting a packet that ends up being frontrun.
- **Data Integrity.** There's no use being the first to submit a packet if the packet is submitted
incorrectly, and a good relayer will never drop packets. The latter is especially important for
ordered channels, as a channel will be closed if a packet on it times out.
- **Quick Startup Times.** RPCs are unreliable, and it's incredibly difficult to build around every
possible failure case - especially when connecting to multiple different chains. Even with proper
error handling and retry logic, in the event of a crash, startup time should be miniscule (see:
<https://github.com/clemensgg/xion-relayer-postmortem>)

Voyager takes a novel approach to solving these problems. Internally, everything is modeled as a
finite state machine, which is stored in postgres to ensure transactional integrity. Every chain
query, transaction submission, and even the data itself is represented as a state within the queue.
This design solves two of the properties mentioned above out of the box: **Data Integrity** and
**Quick Startup Times**. Since no state is stored in Voyager itself, it is able to crash and restart
exactly where it left off, making startup times lightning fast; and since every message is processed
within a postgres transaction, we are guaranteed that the data we're working with is correct
(barring a bug in the business logic, of course). The final property, **Speed**, is also solved
by this design - since each message fully encapsulates all the state it needs to operate, multiple
messages can safely be executed in parallel. This means, for instance, that while one worker is
fetching events from a block, another could be submitting a light client update, and another could
be generating a state proof, and so on.

## Light Clients

Voyager implements the relaying logic for several light client protocols. Some of these are existing
specifications, such as tendermint and ethereum, but most are custom implementations for chains
Union has connected to IBC.

Many of these custom light clients build off of our existing light client infrastructure for
ethereum and tendermint, with additional logic specific to the protocol's finality.

### L2 Clients

Voyager supports several Ethereum L2s, which are implemented as conditional light clients (TODO:
link to our doc on conditional clients)

The settlement condition and update logic is different for each L2, but the basic principles are
the same:

- Verify the account root of the L2 rollup contract on the L1
- Verify the L2 state is stored in the L2 rollup contract
- Verify the IBC account root against the rollup root

The consensus height of L2 clients is the consensus height of the L1 (which in the case of Ethereum
L2s is the height of the beacon chain), and L2 clients are can only be updated to heights that
the L1 client it tracks has a consensus state for. As such, the finality time for L2s is the L2
settlement period + L1 finality time.
