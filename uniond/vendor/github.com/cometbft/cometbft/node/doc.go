/*
Package node is the main entry point, where the Node struct, which
represents a full node, is defined.

Adding new p2p.Reactor(s)

To add a new p2p.Reactor, use the CustomReactors option:

	node, err := NewNode(
			config,
			privVal,
			nodeKey,
			clientCreator,
			genesisDocProvider,
			dbProvider,
			metricsProvider,
			logger,
			CustomReactors(map[string]p2p.Reactor{"CUSTOM": customReactor}),
	)

Replacing existing p2p.Reactor(s)

To replace the built-in p2p.Reactor, use the CustomReactors option:

	node, err := NewNode(
			config,
			privVal,
			nodeKey,
			clientCreator,
			genesisDocProvider,
			dbProvider,
			metricsProvider,
			logger,
			CustomReactors(map[string]p2p.Reactor{"BLOCKSYNC": customBlocksyncReactor}),
	)

The list of existing reactors can be found in CustomReactors documentation.
*/
package node
