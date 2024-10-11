package appdata

// BatchablePacket is the interface that packet types which can be batched implement.
// All types that implement Packet except CommitData also implement BatchablePacket.
// CommitData should not be batched because it forces synchronization of asynchronous listeners.
type BatchablePacket interface {
	Packet
	isBatchablePacket()
}

// PacketBatch is a batch of packets that can be sent to a listener.
// If listener processing is asynchronous, the batch of packets will be sent
// all at once in a single operation which can be more efficient than sending
// each packet individually.
type PacketBatch []BatchablePacket

func (p PacketBatch) apply(l *Listener) error {
	if l.onBatch != nil {
		return l.onBatch(p)
	}

	for _, packet := range p {
		if err := packet.apply(l); err != nil {
			return err
		}
	}

	return nil
}

func (ModuleInitializationData) isBatchablePacket() {}

func (StartBlockData) isBatchablePacket() {}

func (TxData) isBatchablePacket() {}

func (EventData) isBatchablePacket() {}

func (KVPairData) isBatchablePacket() {}

func (ObjectUpdateData) isBatchablePacket() {}
