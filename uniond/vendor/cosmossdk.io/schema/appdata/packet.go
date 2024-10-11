package appdata

// Packet is the interface that all listener data structures implement so that this data can be "packetized"
// and processed in a stream, possibly asynchronously.
// Valid implementations are ModuleInitializationData, StartBlockData, TxData, EventData, KVPairData, ObjectUpdateData,
// and CommitData.
type Packet interface {
	apply(*Listener) error
}

// SendPacket sends a packet to a listener invoking the appropriate callback for this packet if one is registered.
func (l Listener) SendPacket(p Packet) error {
	return p.apply(&l)
}

func (m ModuleInitializationData) apply(l *Listener) error {
	if l.InitializeModuleData == nil {
		return nil
	}
	return l.InitializeModuleData(m)
}

func (b StartBlockData) apply(l *Listener) error {
	if l.StartBlock == nil {
		return nil
	}
	return l.StartBlock(b)
}

func (t TxData) apply(l *Listener) error {
	if l.OnTx == nil {
		return nil
	}
	return l.OnTx(t)
}

func (e EventData) apply(l *Listener) error {
	if l.OnEvent == nil {
		return nil
	}
	return l.OnEvent(e)
}

func (k KVPairData) apply(l *Listener) error {
	if l.OnKVPair == nil {
		return nil
	}
	return l.OnKVPair(k)
}

func (o ObjectUpdateData) apply(l *Listener) error {
	if l.OnObjectUpdate == nil {
		return nil
	}
	return l.OnObjectUpdate(o)
}

func (c CommitData) apply(l *Listener) error {
	if l.Commit == nil {
		return nil
	}
	cb, err := l.Commit(c)
	if err != nil {
		return err
	}
	if cb != nil {
		return cb()
	}
	return nil
}
