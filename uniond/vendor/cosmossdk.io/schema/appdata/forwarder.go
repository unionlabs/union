package appdata

// PacketForwarder creates a listener which listens to all callbacks and forwards all packets to the provided
// function. This is intended to be used primarily for tests and debugging.
func PacketForwarder(f func(Packet) error) Listener {
	return Listener{
		InitializeModuleData: func(data ModuleInitializationData) error { return f(data) },
		OnTx:                 func(data TxData) error { return f(data) },
		OnEvent:              func(data EventData) error { return f(data) },
		OnKVPair:             func(data KVPairData) error { return f(data) },
		OnObjectUpdate:       func(data ObjectUpdateData) error { return f(data) },
		StartBlock:           func(data StartBlockData) error { return f(data) },
		Commit:               func(data CommitData) (func() error, error) { return nil, f(data) },
	}
}
