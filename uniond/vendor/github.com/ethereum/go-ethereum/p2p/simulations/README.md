# devp2p Simulations

The `p2p/simulations` package implements a simulation framework that supports
creating a collection of devp2p nodes, connecting them to form a
simulation network, performing simulation actions in that network and then
extracting useful information.

## Nodes

Each node in a simulation network runs multiple services by wrapping a collection
of objects which implement the `node.Service` interface meaning they:

* can be started and stopped
* run p2p protocols
* expose RPC APIs

This means that any object which implements the `node.Service` interface can be
used to run a node in the simulation.

## Services

Before running a simulation, a set of service initializers must be registered
which can then be used to run nodes in the network.

A service initializer is a function with the following signature:

```go
func(ctx *adapters.ServiceContext) (node.Service, error)
```

These initializers should be registered by calling the `adapters.RegisterServices`
function in an `init()` hook:

```go
func init() {
	adapters.RegisterServices(adapters.Services{
		"service1": initService1,
		"service2": initService2,
	})
}
```

## Node Adapters

The simulation framework includes multiple "node adapters" which are
responsible for creating an environment in which a node runs.

### SimAdapter

The `SimAdapter` runs nodes in-memory, connecting them using an in-memory,
synchronous `net.Pipe` and connecting to their RPC server using an in-memory
`rpc.Client`.

### ExecAdapter

The `ExecAdapter` runs nodes as child processes of the running simulation.

It does this by executing the binary which is running the simulation but
setting `argv[0]` (i.e. the program name) to `p2p-node` which is then
detected by an init hook in the child process which runs the `node.Service`
using the devp2p node stack rather than executing `main()`.

The nodes listen for devp2p connections and WebSocket RPC clients on random
localhost ports.

## Network

A simulation network is created with an ID and default service. The default
service is used if a node is created without an explicit service. The 
network has exposed methods for creating, starting, stopping, connecting 
and disconnecting nodes. It also emits events when certain actions occur.

### Events

A simulation network emits the following events:

* node event       - when nodes are created / started / stopped
* connection event - when nodes are connected / disconnected
* message event    - when a protocol message is sent between two nodes

The events have a "control" flag which when set indicates that the event is the
outcome of a controlled simulation action (e.g. creating a node or explicitly
connecting two nodes).

This is in contrast to a non-control event, otherwise called a "live" event,
which is the outcome of something happening in the network as a result of a
control event (e.g. a node actually started up or a connection was actually
established between two nodes).

Live events are detected by the simulation network by subscribing to node peer
events via RPC when the nodes start up.

## Testing Framework

The `Simulation` type can be used in tests to perform actions in a simulation
network and then wait for expectations to be met.

With a running simulation network, the `Simulation.Run` method can be called
with a `Step` which has the following fields:

* `Action` - a function that performs some action in the network

* `Expect` - an expectation function which returns whether or not a
    given node meets the expectation

* `Trigger` - a channel that receives node IDs which then trigger a check
    of the expectation function to be performed against that node

As a concrete example, consider a simulated network of Ethereum nodes. An
`Action` could be the sending of a transaction, `Expect` it being included in
a block, and `Trigger` a check for every block that is mined.

On return, the `Simulation.Run` method returns a `StepResult` which can be used
to determine if all nodes met the expectation, how long it took them to meet
the expectation and what network events were emitted during the step run.

## HTTP API

The simulation framework includes a HTTP API that can be used to control the
simulation.

The API is initialised with a particular node adapter and has the following
endpoints:

```
GET    /                            Get network information
POST   /start                       Start all nodes in the network
POST   /stop                        Stop all nodes in the network
GET    /events                      Stream network events
GET    /snapshot                    Take a network snapshot
POST   /snapshot                    Load a network snapshot
POST   /nodes                       Create a node
GET    /nodes                       Get all nodes in the network
GET    /nodes/:nodeid               Get node information
POST   /nodes/:nodeid/start         Start a node
POST   /nodes/:nodeid/stop          Stop a node
POST   /nodes/:nodeid/conn/:peerid  Connect two nodes
DELETE /nodes/:nodeid/conn/:peerid  Disconnect two nodes
GET    /nodes/:nodeid/rpc           Make RPC requests to a node via WebSocket
```

For convenience, `nodeid` in the URL can be the name of a node rather than its
ID.

## Command line client

`p2psim` is a command line client for the HTTP API, located in
`cmd/p2psim`.

It provides the following commands:

```
p2psim show
p2psim events [--current] [--filter=FILTER]
p2psim snapshot
p2psim load
p2psim node create [--name=NAME] [--services=SERVICES] [--key=KEY]
p2psim node list
p2psim node show <node>
p2psim node start <node>
p2psim node stop <node>
p2psim node connect <node> <peer>
p2psim node disconnect <node> <peer>
p2psim node rpc <node> <method> [<args>] [--subscribe]
```

## Example

See [p2p/simulations/examples/README.md](examples/README.md).
