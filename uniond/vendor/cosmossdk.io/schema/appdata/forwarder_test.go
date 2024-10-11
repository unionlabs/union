package appdata

import (
	"reflect"
	"testing"
)

func TestPacketForwarder(t *testing.T) {
	var received []Packet
	listener := PacketForwarder(func(packet Packet) error {
		received = append(received, packet)
		return nil
	})

	expected := []Packet{
		ModuleInitializationData{},
		StartBlockData{},
		TxData{},
		EventData{},
		KVPairData{},
		ObjectUpdateData{},
		CommitData{},
	}

	for i, packet := range expected {
		err := listener.SendPacket(packet)
		if err != nil {
			t.Fatal(err)
		}

		if len(received) != i+1 {
			t.Fatalf("didn't receive packet %v", packet)
		}

		if !reflect.DeepEqual(received[i], packet) {
			t.Fatalf("received packet %v, expected %v", received[i], packet)
		}
	}
}
