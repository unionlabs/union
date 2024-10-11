// Copyright 2020 The go-ethereum Authors
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

package nodestate

import (
	"errors"
	"fmt"
	"reflect"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/ethereum/go-ethereum/rlp"
)

func testSetup(flagPersist []bool, fieldType []reflect.Type) (*Setup, []Flags, []Field) {
	setup := &Setup{}
	flags := make([]Flags, len(flagPersist))
	for i, persist := range flagPersist {
		if persist {
			flags[i] = setup.NewPersistentFlag(fmt.Sprintf("flag-%d", i))
		} else {
			flags[i] = setup.NewFlag(fmt.Sprintf("flag-%d", i))
		}
	}
	fields := make([]Field, len(fieldType))
	for i, ftype := range fieldType {
		switch ftype {
		case reflect.TypeOf(uint64(0)):
			fields[i] = setup.NewPersistentField(fmt.Sprintf("field-%d", i), ftype, uint64FieldEnc, uint64FieldDec)
		case reflect.TypeOf(""):
			fields[i] = setup.NewPersistentField(fmt.Sprintf("field-%d", i), ftype, stringFieldEnc, stringFieldDec)
		default:
			fields[i] = setup.NewField(fmt.Sprintf("field-%d", i), ftype)
		}
	}
	return setup, flags, fields
}

func testNode(b byte) *enode.Node {
	r := &enr.Record{}
	r.SetSig(dummyIdentity{b}, []byte{42})
	n, _ := enode.New(dummyIdentity{b}, r)
	return n
}

func TestCallback(t *testing.T) {
	mdb, clock := rawdb.NewMemoryDatabase(), &mclock.Simulated{}

	s, flags, _ := testSetup([]bool{false, false, false}, nil)
	ns := NewNodeStateMachine(mdb, []byte("-ns"), clock, s)

	set0 := make(chan struct{}, 1)
	set1 := make(chan struct{}, 1)
	set2 := make(chan struct{}, 1)
	ns.SubscribeState(flags[0], func(n *enode.Node, oldState, newState Flags) { set0 <- struct{}{} })
	ns.SubscribeState(flags[1], func(n *enode.Node, oldState, newState Flags) { set1 <- struct{}{} })
	ns.SubscribeState(flags[2], func(n *enode.Node, oldState, newState Flags) { set2 <- struct{}{} })

	ns.Start()

	ns.SetState(testNode(1), flags[0], Flags{}, 0)
	ns.SetState(testNode(1), flags[1], Flags{}, time.Second)
	ns.SetState(testNode(1), flags[2], Flags{}, 2*time.Second)

	for i := 0; i < 3; i++ {
		select {
		case <-set0:
		case <-set1:
		case <-set2:
		case <-time.After(time.Second):
			t.Fatalf("failed to invoke callback")
		}
	}
}

func TestPersistentFlags(t *testing.T) {
	mdb, clock := rawdb.NewMemoryDatabase(), &mclock.Simulated{}

	s, flags, _ := testSetup([]bool{true, true, true, false}, nil)
	ns := NewNodeStateMachine(mdb, []byte("-ns"), clock, s)

	saveNode := make(chan *nodeInfo, 5)
	ns.saveNodeHook = func(node *nodeInfo) {
		saveNode <- node
	}

	ns.Start()

	ns.SetState(testNode(1), flags[0], Flags{}, time.Second) // state with timeout should not be saved
	ns.SetState(testNode(2), flags[1], Flags{}, 0)
	ns.SetState(testNode(3), flags[2], Flags{}, 0)
	ns.SetState(testNode(4), flags[3], Flags{}, 0)
	ns.SetState(testNode(5), flags[0], Flags{}, 0)
	ns.Persist(testNode(5))
	select {
	case <-saveNode:
	case <-time.After(time.Second):
		t.Fatalf("Timeout")
	}
	ns.Stop()

	for i := 0; i < 2; i++ {
		select {
		case <-saveNode:
		case <-time.After(time.Second):
			t.Fatalf("Timeout")
		}
	}
	select {
	case <-saveNode:
		t.Fatalf("Unexpected saveNode")
	case <-time.After(time.Millisecond * 100):
	}
}

func TestSetField(t *testing.T) {
	mdb, clock := rawdb.NewMemoryDatabase(), &mclock.Simulated{}

	s, flags, fields := testSetup([]bool{true}, []reflect.Type{reflect.TypeOf("")})
	ns := NewNodeStateMachine(mdb, []byte("-ns"), clock, s)

	saveNode := make(chan *nodeInfo, 1)
	ns.saveNodeHook = func(node *nodeInfo) {
		saveNode <- node
	}

	ns.Start()

	// Set field before setting state
	ns.SetField(testNode(1), fields[0], "hello world")
	field := ns.GetField(testNode(1), fields[0])
	if field == nil {
		t.Fatalf("Field should be set before setting states")
	}
	ns.SetField(testNode(1), fields[0], nil)
	field = ns.GetField(testNode(1), fields[0])
	if field != nil {
		t.Fatalf("Field should be unset")
	}
	// Set field after setting state
	ns.SetState(testNode(1), flags[0], Flags{}, 0)
	ns.SetField(testNode(1), fields[0], "hello world")
	field = ns.GetField(testNode(1), fields[0])
	if field == nil {
		t.Fatalf("Field should be set after setting states")
	}
	if err := ns.SetField(testNode(1), fields[0], 123); err == nil {
		t.Fatalf("Invalid field should be rejected")
	}
	// Dirty node should be written back
	ns.Stop()
	select {
	case <-saveNode:
	case <-time.After(time.Second):
		t.Fatalf("Timeout")
	}
}

func TestSetState(t *testing.T) {
	mdb, clock := rawdb.NewMemoryDatabase(), &mclock.Simulated{}

	s, flags, _ := testSetup([]bool{false, false, false}, nil)
	ns := NewNodeStateMachine(mdb, []byte("-ns"), clock, s)

	type change struct{ old, new Flags }
	set := make(chan change, 1)
	ns.SubscribeState(flags[0].Or(flags[1]), func(n *enode.Node, oldState, newState Flags) {
		set <- change{
			old: oldState,
			new: newState,
		}
	})

	ns.Start()

	check := func(expectOld, expectNew Flags, expectChange bool) {
		if expectChange {
			select {
			case c := <-set:
				if !c.old.Equals(expectOld) {
					t.Fatalf("Old state mismatch")
				}
				if !c.new.Equals(expectNew) {
					t.Fatalf("New state mismatch")
				}
			case <-time.After(time.Second):
			}
			return
		}
		select {
		case <-set:
			t.Fatalf("Unexpected change")
		case <-time.After(time.Millisecond * 100):
			return
		}
	}
	ns.SetState(testNode(1), flags[0], Flags{}, 0)
	check(Flags{}, flags[0], true)

	ns.SetState(testNode(1), flags[1], Flags{}, 0)
	check(flags[0], flags[0].Or(flags[1]), true)

	ns.SetState(testNode(1), flags[2], Flags{}, 0)
	check(Flags{}, Flags{}, false)

	ns.SetState(testNode(1), Flags{}, flags[0], 0)
	check(flags[0].Or(flags[1]), flags[1], true)

	ns.SetState(testNode(1), Flags{}, flags[1], 0)
	check(flags[1], Flags{}, true)

	ns.SetState(testNode(1), Flags{}, flags[2], 0)
	check(Flags{}, Flags{}, false)

	ns.SetState(testNode(1), flags[0].Or(flags[1]), Flags{}, time.Second)
	check(Flags{}, flags[0].Or(flags[1]), true)
	clock.Run(time.Second)
	check(flags[0].Or(flags[1]), Flags{}, true)
}

func uint64FieldEnc(field interface{}) ([]byte, error) {
	if u, ok := field.(uint64); ok {
		enc, err := rlp.EncodeToBytes(&u)
		return enc, err
	}
	return nil, errors.New("invalid field type")
}

func uint64FieldDec(enc []byte) (interface{}, error) {
	var u uint64
	err := rlp.DecodeBytes(enc, &u)
	return u, err
}

func stringFieldEnc(field interface{}) ([]byte, error) {
	if s, ok := field.(string); ok {
		return []byte(s), nil
	}
	return nil, errors.New("invalid field type")
}

func stringFieldDec(enc []byte) (interface{}, error) {
	return string(enc), nil
}

func TestPersistentFields(t *testing.T) {
	mdb, clock := rawdb.NewMemoryDatabase(), &mclock.Simulated{}

	s, flags, fields := testSetup([]bool{true}, []reflect.Type{reflect.TypeOf(uint64(0)), reflect.TypeOf("")})
	ns := NewNodeStateMachine(mdb, []byte("-ns"), clock, s)

	ns.Start()
	ns.SetState(testNode(1), flags[0], Flags{}, 0)
	ns.SetField(testNode(1), fields[0], uint64(100))
	ns.SetField(testNode(1), fields[1], "hello world")
	ns.Stop()

	ns2 := NewNodeStateMachine(mdb, []byte("-ns"), clock, s)

	ns2.Start()
	field0 := ns2.GetField(testNode(1), fields[0])
	if !reflect.DeepEqual(field0, uint64(100)) {
		t.Fatalf("Field changed")
	}
	field1 := ns2.GetField(testNode(1), fields[1])
	if !reflect.DeepEqual(field1, "hello world") {
		t.Fatalf("Field changed")
	}

	s.Version++
	ns3 := NewNodeStateMachine(mdb, []byte("-ns"), clock, s)
	ns3.Start()
	if ns3.GetField(testNode(1), fields[0]) != nil {
		t.Fatalf("Old field version should have been discarded")
	}
}

func TestFieldSub(t *testing.T) {
	mdb, clock := rawdb.NewMemoryDatabase(), &mclock.Simulated{}

	s, flags, fields := testSetup([]bool{true}, []reflect.Type{reflect.TypeOf(uint64(0))})
	ns := NewNodeStateMachine(mdb, []byte("-ns"), clock, s)

	var (
		lastState                  Flags
		lastOldValue, lastNewValue interface{}
	)
	ns.SubscribeField(fields[0], func(n *enode.Node, state Flags, oldValue, newValue interface{}) {
		lastState, lastOldValue, lastNewValue = state, oldValue, newValue
	})
	check := func(state Flags, oldValue, newValue interface{}) {
		if !lastState.Equals(state) || lastOldValue != oldValue || lastNewValue != newValue {
			t.Fatalf("Incorrect field sub callback (expected [%v %v %v], got [%v %v %v])", state, oldValue, newValue, lastState, lastOldValue, lastNewValue)
		}
	}
	ns.Start()
	ns.SetState(testNode(1), flags[0], Flags{}, 0)
	ns.SetField(testNode(1), fields[0], uint64(100))
	check(flags[0], nil, uint64(100))
	ns.Stop()
	check(s.OfflineFlag(), uint64(100), nil)

	ns2 := NewNodeStateMachine(mdb, []byte("-ns"), clock, s)
	ns2.SubscribeField(fields[0], func(n *enode.Node, state Flags, oldValue, newValue interface{}) {
		lastState, lastOldValue, lastNewValue = state, oldValue, newValue
	})
	ns2.Start()
	check(s.OfflineFlag(), nil, uint64(100))
	ns2.SetState(testNode(1), Flags{}, flags[0], 0)
	ns2.SetField(testNode(1), fields[0], nil)
	check(Flags{}, uint64(100), nil)
	ns2.Stop()
}

func TestDuplicatedFlags(t *testing.T) {
	mdb, clock := rawdb.NewMemoryDatabase(), &mclock.Simulated{}

	s, flags, _ := testSetup([]bool{true}, nil)
	ns := NewNodeStateMachine(mdb, []byte("-ns"), clock, s)

	type change struct{ old, new Flags }
	set := make(chan change, 1)
	ns.SubscribeState(flags[0], func(n *enode.Node, oldState, newState Flags) {
		set <- change{oldState, newState}
	})

	ns.Start()
	defer ns.Stop()

	check := func(expectOld, expectNew Flags, expectChange bool) {
		if expectChange {
			select {
			case c := <-set:
				if !c.old.Equals(expectOld) {
					t.Fatalf("Old state mismatch")
				}
				if !c.new.Equals(expectNew) {
					t.Fatalf("New state mismatch")
				}
			case <-time.After(time.Second):
			}
			return
		}
		select {
		case <-set:
			t.Fatalf("Unexpected change")
		case <-time.After(time.Millisecond * 100):
			return
		}
	}
	ns.SetState(testNode(1), flags[0], Flags{}, time.Second)
	check(Flags{}, flags[0], true)
	ns.SetState(testNode(1), flags[0], Flags{}, 2*time.Second) // extend the timeout to 2s
	check(Flags{}, flags[0], false)

	clock.Run(2 * time.Second)
	check(flags[0], Flags{}, true)
}

func TestCallbackOrder(t *testing.T) {
	mdb, clock := rawdb.NewMemoryDatabase(), &mclock.Simulated{}

	s, flags, _ := testSetup([]bool{false, false, false, false}, nil)
	ns := NewNodeStateMachine(mdb, []byte("-ns"), clock, s)

	ns.SubscribeState(flags[0], func(n *enode.Node, oldState, newState Flags) {
		if newState.Equals(flags[0]) {
			ns.SetStateSub(n, flags[1], Flags{}, 0)
			ns.SetStateSub(n, flags[2], Flags{}, 0)
		}
	})
	ns.SubscribeState(flags[1], func(n *enode.Node, oldState, newState Flags) {
		if newState.Equals(flags[1]) {
			ns.SetStateSub(n, flags[3], Flags{}, 0)
		}
	})
	lastState := Flags{}
	ns.SubscribeState(MergeFlags(flags[1], flags[2], flags[3]), func(n *enode.Node, oldState, newState Flags) {
		if !oldState.Equals(lastState) {
			t.Fatalf("Wrong callback order")
		}
		lastState = newState
	})

	ns.Start()
	defer ns.Stop()

	ns.SetState(testNode(1), flags[0], Flags{}, 0)
}
