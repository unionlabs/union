// -*- mode: Bluespec; -*-
/*
 * Reactor is responsible for handling incoming messages on one or more
 * Channel. Switch calls GetChannels when reactor is added to it. When a new
 * peer joins our node, InitPeer and AddPeer are called. RemovePeer is called
 * when the peer is stopped. Receive is called when a message is received on a
 * channel associated with this reactor.
 */
// Code: https://github.com/cometbft/cometbft/blob/main/p2p/base_reactor.go
module reactor {

	// Unique ID of a node.
	type NodeID = str

	/*
	 * Peer is an interface representing a peer connected on a reactor.
	 */
	type Peer = {
		ID: NodeID,

		// Other fields can be added to represent the p2p operation.
	}

	// Byte ID used by channels, must be globally unique.
	type Byte = str

	// Channel configuration.
	type ChannelDescriptor = {
		ID: Byte,
		Priority: int,
	}

	/*
	 * Envelope contains a message with sender routing info.
	 */
	type Envelope = {
		Src: Peer, // Sender
		Message: str, // Payload
		ChannelID: Byte,
	}

	// A Routine is used to interact with an active Peer.
	type Routine = {
		name: str,
		peer: Peer,
	}

	type ReactorState = {
		// Peers that have been initialized but not yet removed.
		// The reactor should expect receiving messages from them.
		peers: Set[NodeID],

		// The reactor runs multiple routines.
		routines: Set[Routine],

		// Values: init -> registered -> running -> stopped
		state: str,

		// Name with which the reactor was registered.
		name: str,

		// Channels the reactor is responsible for.
		channels: Set[ChannelDescriptor],
	}

	// Produces a new, uninitialized reactor.
	pure def NewReactor(): ReactorState = {
		{
			peers: Set(),
			routines: Set(),
			state: "init",
			name: "",
			channels: Set(),
		}
	}

	// Pure definitions below represent the `p2p.Reactor` interface methods:

	/*
	 * GetChannels returns the list of MConnection.ChannelDescriptor. Make sure
	 * that each ID is unique across all the reactors added to the switch.
	 */
	pure def GetChannels(s: ReactorState): Set[ChannelDescriptor] = {
		s.channels // Static list, configured at initialization.
	}

	/*
	 * SetSwitch allows setting a switch.
	 */
	pure def SetSwitch(s: ReactorState, switch: bool): ReactorState = {
		s.with("state", "registered")
	}

	/*
	 * Start the service.
	 * If it's already started or stopped, will return an error.
	 */
	pure def OnStart(s: ReactorState): ReactorState = {
		// Startup procedures should come here.
		s.with("state", "running")
	}

	/*
	 * Stop the service.
	 * If it's already stopped, will return an error.
	 */
	pure def OnStop(s: ReactorState): ReactorState = {
		// Shutdown procedures should come here.
		s.with("state", "stopped")
	}

	/*
	 * InitPeer is called by the switch before the peer is started. Use it to
	 * initialize data for the peer (e.g. peer state).
	 */
	pure def InitPeer(s: ReactorState, peer: Peer): (ReactorState, Peer) = {
		// This method can update the received peer, which is returned.
		val updatedPeer = peer
		(s.with("peers", s.peers.union(Set(peer.ID))), updatedPeer)
	}

	/* 
	 * AddPeer is called by the switch after the peer is added and successfully
	 * started. Use it to start goroutines communicating with the peer.
	 */
	pure def AddPeer(s: ReactorState, peer: Peer): ReactorState = {
		// This method can be used to start routines to handle the peer.
		// Below an example of an arbitrary 'ioRoutine' routine.
		val startedRoutines = Set( {name: "ioRoutine", peer: peer} )
		s.with("routines", s.routines.union(startedRoutines))
	}

	/*
	 * RemovePeer is called by the switch when the peer is stopped (due to error
	 * or other reason).
	 */
	pure def RemovePeer(s: ReactorState, peer: Peer, reason: str): ReactorState = {
		// This method should stop routines created by `AddPeer(Peer)`.
		val stoppedRoutines = s.routines.filter(r => r.peer.ID == peer.ID)
		s.with("peers", s.peers.exclude(Set(peer.ID)))
		 .with("routines", s.routines.exclude(stoppedRoutines))
	}

	/*
	 * Receive is called by the switch when an envelope is received from any connected
	 * peer on any of the channels registered by the reactor.
	 */
	pure def Receive(s: ReactorState, e: Envelope): ReactorState = {
		// This method should process the message payload: e.Message.
		s
	}

	// Global state

	// Reactors are uniquely identified by their names.
	var reactors: str -> ReactorState

	// Reactor (name) assigned to each channel ID.
	var reactorsByCh: Byte -> str

	// Helper action to (only) update the state of a given reactor.
	action updateReactorTo(reactor: ReactorState): bool = all {
		reactors' = reactors.set(reactor.name, reactor),
		reactorsByCh' = reactorsByCh
	}

	// State transitions performed by the p2p layer, invoking `p2p.Reactor` methods:

	// Code: Switch.AddReactor(name string, reactor Reactor)
	action register(name: str, reactor: ReactorState): bool = all {
		reactor.state == "init",
		// Assign the reactor as responsible for its channel IDs, which
		// should not be already assigned to another reactor.
		val chIDs = reactor.GetChannels().map(c => c.ID)
		all {
			size(chIDs.intersect(reactorsByCh.keys())) == 0,
			reactorsByCh' = reactorsByCh.keys().union(chIDs).
					mapBy(id => if (id.in(chIDs)) name
						    else reactorsByCh.get(id)),
		},
		// Register the reactor by its name, which must be unique.
		not(name.in(reactors.keys())),
		reactors' = reactors.put(name,
			reactor.SetSwitch(true).with("name", name))
	}

	// Code: Switch.OnStart()
	action start(reactor: ReactorState): bool = all {
		reactor.state == "registered",
		updateReactorTo(reactor.OnStart())
	}

	// Code: Switch.addPeer(p Peer): preamble
	action initPeer(reactor: ReactorState, peer: Peer): bool = all {
		reactor.state == "running",
		not(peer.ID.in(reactor.peers)),
		updateReactorTo(reactor.InitPeer(peer)._1)
	}

	// Code: Switch.addPeer(p Peer): conclusion
	action addPeer(reactor: ReactorState, peer: Peer): bool = all {
		reactor.state == "running",
		peer.ID.in(reactor.peers), // InitPeer(peer) and not RemovePeer(peer)
		reactor.routines.filter(r => r.peer.ID == peer.ID).size() == 0,
		updateReactorTo(reactor.AddPeer(peer))
	}

	// Code: Switch.stopAndRemovePeer(peer Peer, reason interface{})
	action removePeer(reactor: ReactorState, peer: Peer, reason: str): bool = all {
		reactor.state == "running",
		peer.ID.in(reactor.peers), // InitPeer(peer) and not RemovePeer(peer)
		// Routines might not be started, namely: not AddPeer(peer)
		// Routines could also be already stopped if Peer has erroed.
		updateReactorTo(reactor.RemovePeer(peer, reason))
	}

	// Code: Peer type, onReceive := func(chID byte, msgBytes []byte)
	action receive(reactor: ReactorState, e: Envelope): bool = all {
		reactor.state == "running",
		// The message's sender is an active peer
		e.Src.ID.in(reactor.peers),
		// Reactor is assigned to the message's channel ID
		e.ChannelID.in(reactorsByCh.keys()),
		reactorsByCh.get(e.ChannelID) == reactor.name,
		reactor.GetChannels().exists(c => c.ID == e.ChannelID),
		updateReactorTo(reactor.Receive(e))
	}

	// Code: Switch.OnStop()
	action stop(reactor: ReactorState): bool = all {
		reactor.state == "running",
		// Either no peer was added or all peers were removed
		reactor.peers.size() == 0,
		updateReactorTo(reactor.OnStop())
	}

	// Simulation support

	action init = all {
		reactors' = Map(),
		reactorsByCh' = Map(),
	}

	// Modelled reactor configuration
	pure val reactorName = "myReactor"
	pure val reactorChannels = Set({ID: "3", Priority: 1}, {ID: "7", Priority: 2})

	// For retro-compatibility: the state of the modelled reactor
	def state(): ReactorState = {
		reactors.get(reactorName)
	}

	pure val samplePeers = Set({ID: "p1"}, {ID: "p3"})
	pure val sampleChIDs = Set("1", "3", "7") // ChannelID 1 not registered
	pure val sampleMsgs  = Set("ping", "pong")

	action step = any {
		register(reactorName, NewReactor.with("channels", reactorChannels)),
		val reactor = reactors.get(reactorName)
		any {
			reactor.start(),
			reactor.stop(),
			nondet peer = oneOf(samplePeers)
			any {
				// Peer-specific actions
				reactor.initPeer(peer),
				reactor.addPeer(peer),
				reactor.removePeer(peer, "no reason"),
				reactor.receive({Src: peer,
					ChannelID: oneOf(sampleChIDs),
					Message: oneOf(sampleMsgs)}),
			}
		}
	}

}
