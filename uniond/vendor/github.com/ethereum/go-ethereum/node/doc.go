// Copyright 2016 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

/*
Package node sets up multi-protocol Ethereum nodes.

In the model exposed by this package, a node is a collection of services which use shared
resources to provide RPC APIs. Services can also offer devp2p protocols, which are wired
up to the devp2p network when the node instance is started.

# Node Lifecycle

The Node object has a lifecycle consisting of three basic states, INITIALIZING, RUNNING
and CLOSED.

	●───────┐
	     New()
	        │
	        ▼
	  INITIALIZING ────Start()─┐
	        │                  │
	        │                  ▼
	    Close()             RUNNING
	        │                  │
	        ▼                  │
	     CLOSED ◀──────Close()─┘

Creating a Node allocates basic resources such as the data directory and returns the node
in its INITIALIZING state. Lifecycle objects, RPC APIs and peer-to-peer networking
protocols can be registered in this state. Basic operations such as opening a key-value
database are permitted while initializing.

Once everything is registered, the node can be started, which moves it into the RUNNING
state. Starting the node starts all registered Lifecycle objects and enables RPC and
peer-to-peer networking. Note that no additional Lifecycles, APIs or p2p protocols can be
registered while the node is running.

Closing the node releases all held resources. The actions performed by Close depend on the
state it was in. When closing a node in INITIALIZING state, resources related to the data
directory are released. If the node was RUNNING, closing it also stops all Lifecycle
objects and shuts down RPC and peer-to-peer networking.

You must always call Close on Node, even if the node was not started.

# Resources Managed By Node

All file-system resources used by a node instance are located in a directory called the
data directory. The location of each resource can be overridden through additional node
configuration. The data directory is optional. If it is not set and the location of a
resource is otherwise unspecified, package node will create the resource in memory.

To access to the devp2p network, Node configures and starts p2p.Server. Each host on the
devp2p network has a unique identifier, the node key. The Node instance persists this key
across restarts. Node also loads static and trusted node lists and ensures that knowledge
about other hosts is persisted.

JSON-RPC servers which run HTTP, WebSocket or IPC can be started on a Node. RPC modules
offered by registered services will be offered on those endpoints. Users can restrict any
endpoint to a subset of RPC modules. Node itself offers the "debug", "admin" and "web3"
modules.

Service implementations can open LevelDB databases through the service context. Package
node chooses the file system location of each database. If the node is configured to run
without a data directory, databases are opened in memory instead.

Node also creates the shared store of encrypted Ethereum account keys. Services can access
the account manager through the service context.

# Sharing Data Directory Among Instances

Multiple node instances can share a single data directory if they have distinct instance
names (set through the Name config option). Sharing behaviour depends on the type of
resource.

devp2p-related resources (node key, static/trusted node lists, known hosts database) are
stored in a directory with the same name as the instance. Thus, multiple node instances
using the same data directory will store this information in different subdirectories of
the data directory.

LevelDB databases are also stored within the instance subdirectory. If multiple node
instances use the same data directory, opening the databases with identical names will
create one database for each instance.

The account key store is shared among all node instances using the same data directory
unless its location is changed through the KeyStoreDir configuration option.

# Data Directory Sharing Example

In this example, two node instances named A and B are started with the same data
directory. Node instance A opens the database "db", node instance B opens the databases
"db" and "db-2". The following files will be created in the data directory:

	data-directory/
		A/
			nodekey            -- devp2p node key of instance A
			nodes/             -- devp2p discovery knowledge database of instance A
			db/                -- LevelDB content for "db"
		A.ipc                  -- JSON-RPC UNIX domain socket endpoint of instance A
		B/
			nodekey            -- devp2p node key of node B
			nodes/             -- devp2p discovery knowledge database of instance B
			static-nodes.json  -- devp2p static node list of instance B
			db/                -- LevelDB content for "db"
			db-2/              -- LevelDB content for "db-2"
		B.ipc                  -- JSON-RPC UNIX domain socket endpoint of instance B
		keystore/              -- account key store, used by both instances
*/
package node
