package appdata

// ListenerMux returns a listener that forwards received events to all the provided listeners in order.
// A callback is only registered if a non-nil callback is present in at least one of the listeners.
func ListenerMux(listeners ...Listener) Listener {
	mux := Listener{}

	initModDataCbs := make([]func(ModuleInitializationData) error, 0, len(listeners))
	for _, l := range listeners {
		if l.InitializeModuleData != nil {
			initModDataCbs = append(initModDataCbs, l.InitializeModuleData)
		}
	}
	if len(initModDataCbs) > 0 {
		mux.InitializeModuleData = func(data ModuleInitializationData) error {
			for _, cb := range initModDataCbs {
				if err := cb(data); err != nil {
					return err
				}
			}
			return nil
		}
	}

	startBlockCbs := make([]func(StartBlockData) error, 0, len(listeners))
	for _, l := range listeners {
		if l.StartBlock != nil {
			startBlockCbs = append(startBlockCbs, l.StartBlock)
		}
	}
	if len(startBlockCbs) > 0 {
		mux.StartBlock = func(data StartBlockData) error {
			for _, cb := range startBlockCbs {
				if err := cb(data); err != nil {
					return err
				}
			}
			return nil
		}
	}

	onTxCbs := make([]func(TxData) error, 0, len(listeners))
	for _, l := range listeners {
		if l.OnTx != nil {
			onTxCbs = append(onTxCbs, l.OnTx)
		}
	}
	if len(onTxCbs) > 0 {
		mux.OnTx = func(data TxData) error {
			for _, cb := range onTxCbs {
				if err := cb(data); err != nil {
					return err
				}
			}
			return nil
		}
	}

	onEventCbs := make([]func(EventData) error, 0, len(listeners))
	for _, l := range listeners {
		if l.OnEvent != nil {
			onEventCbs = append(onEventCbs, l.OnEvent)
		}
	}
	if len(onEventCbs) > 0 {
		mux.OnEvent = func(data EventData) error {
			for _, cb := range onEventCbs {
				if err := cb(data); err != nil {
					return err
				}
			}
			return nil
		}
	}

	onKvPairCbs := make([]func(KVPairData) error, 0, len(listeners))
	for _, l := range listeners {
		if l.OnKVPair != nil {
			onKvPairCbs = append(onKvPairCbs, l.OnKVPair)
		}
	}
	if len(onKvPairCbs) > 0 {
		mux.OnKVPair = func(data KVPairData) error {
			for _, cb := range onKvPairCbs {
				if err := cb(data); err != nil {
					return err
				}
			}
			return nil
		}
	}

	onObjectUpdateCbs := make([]func(ObjectUpdateData) error, 0, len(listeners))
	for _, l := range listeners {
		if l.OnObjectUpdate != nil {
			onObjectUpdateCbs = append(onObjectUpdateCbs, l.OnObjectUpdate)
		}
	}
	if len(onObjectUpdateCbs) > 0 {
		mux.OnObjectUpdate = func(data ObjectUpdateData) error {
			for _, cb := range onObjectUpdateCbs {
				if err := cb(data); err != nil {
					return err
				}
			}
			return nil
		}
	}

	commitCbs := make([]func(CommitData) (func() error, error), 0, len(listeners))
	for _, l := range listeners {
		if l.Commit != nil {
			commitCbs = append(commitCbs, l.Commit)
		}
	}
	n := len(commitCbs)
	if n > 0 {
		mux.Commit = func(data CommitData) (func() error, error) {
			waitCbs := make([]func() error, 0, n)
			for _, cb := range commitCbs {
				wait, err := cb(data)
				if err != nil {
					return nil, err
				}
				if wait != nil {
					waitCbs = append(waitCbs, wait)
				}
			}
			return func() error {
				for _, cb := range waitCbs {
					if err := cb(); err != nil {
						return err
					}
				}
				return nil
			}, nil
		}
	}

	mux.onBatch = func(batch PacketBatch) error {
		for _, listener := range listeners {
			err := batch.apply(&listener) //nolint:gosec // aliasing is safe here
			if err != nil {
				return err
			}
		}
		return nil
	}

	return mux
}
