package dbus

import (
	"context"
	"testing"
	"time"
)

type objectGoContextServer struct {
	t     *testing.T
	sleep time.Duration
}

func (o objectGoContextServer) Sleep() *Error {
	o.t.Log("Got object call and sleeping for ", o.sleep)
	time.Sleep(o.sleep)
	o.t.Log("Completed sleeping for ", o.sleep)
	return nil
}

func TestObjectGoWithContextTimeout(t *testing.T) {
	bus, err := SessionBus()
	if err != nil {
		t.Fatalf("Unexpected error connecting to session bus: %s", err)
	}

	name := bus.Names()[0]
	bus.Export(objectGoContextServer{t, time.Second}, "/org/dannin/DBus/Test", "org.dannin.DBus.Test")
	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()
	select {
	case call := <-bus.Object(name, "/org/dannin/DBus/Test").GoWithContext(ctx, "org.dannin.DBus.Test.Sleep", 0, nil).Done:
		if call.Err != ctx.Err() {
			t.Fatal("Expected ", ctx.Err(), " but got ", call.Err)
		}
	case <-time.After(2 * time.Second):
		t.Fatal("Expected call to not respond in time")
	}
}

func TestObjectGoWithContext(t *testing.T) {
	bus, err := SessionBus()
	if err != nil {
		t.Fatalf("Unexpected error connecting to session bus: %s", err)
	}

	name := bus.Names()[0]
	bus.Export(objectGoContextServer{t, time.Millisecond}, "/org/dannin/DBus/Test", "org.dannin.DBus.Test")
	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()
	select {
	case call := <-bus.Object(name, "/org/dannin/DBus/Test").GoWithContext(ctx, "org.dannin.DBus.Test.Sleep", 0, nil).Done:
		if call.Err != ctx.Err() {
			t.Fatal("Expected ", ctx.Err(), " but got ", call.Err)
		}
	case <-time.After(time.Second):
		t.Fatal("Expected call to respond in 1 Millisecond")
	}
}

type nopServer struct{}

func (_ nopServer) Nop() *Error {
	return nil
}

func fetchSignal(t *testing.T, ch chan *Signal, timeout time.Duration) *Signal {
	select {
	case sig := <-ch:
		return sig
	case <-time.After(timeout):
		t.Fatalf("Failed to fetch signal in specified timeout %s", timeout)
	}
	return nil
}

func TestObjectSignalHandling(t *testing.T) {
	bus, err := SessionBus()
	if err != nil {
		t.Fatalf("Unexpected error connecting to session bus: %s", err)
	}

	name := bus.Names()[0]
	path := ObjectPath("/org/godbus/DBus/TestSignals")
	otherPath := ObjectPath("/org/other-godbus/DBus/TestSignals")
	iface := "org.godbus.DBus.TestSignals"
	otherIface := "org.godbus.DBus.OtherTestSignals"
	err = bus.Export(nopServer{}, path, iface)
	if err != nil {
		t.Fatalf("Unexpected error registering nop server: %v", err)
	}

	obj := bus.Object(name, path)
	if err := bus.AddMatchSignal(
		WithMatchInterface(iface),
		WithMatchMember("Heartbeat"),
		WithMatchObjectPath(path),
	); err != nil {
		t.Fatal(err)
	}

	ch := make(chan *Signal, 5)
	bus.Signal(ch)

	go func() {
		defer func() {
			if err := recover(); err != nil {
				t.Errorf("Catched panic in emitter goroutine: %v", err)
			}
		}()

		// desired signals
		bus.Emit(path, iface+".Heartbeat", uint32(1))
		bus.Emit(path, iface+".Heartbeat", uint32(2))
		// undesired signals
		bus.Emit(otherPath, iface+".Heartbeat", uint32(3))
		bus.Emit(otherPath, otherIface+".Heartbeat", uint32(4))
		bus.Emit(path, iface+".Updated", false)
		// sentinel
		bus.Emit(path, iface+".Heartbeat", uint32(5))

		time.Sleep(100 * time.Millisecond)
		bus.Emit(path, iface+".Heartbeat", uint32(6))
	}()

	checkSignal := func(sig *Signal, value uint32) {
		if sig.Path != path {
			t.Errorf("signal.Path mismatch: %s != %s", path, sig.Path)
		}

		name := iface + ".Heartbeat"
		if sig.Name != name {
			t.Errorf("signal.Name mismatch: %s != %s", name, sig.Name)
		}

		if len(sig.Body) != 1 {
			t.Errorf("Invalid signal body length: %d", len(sig.Body))
			return
		}

		if sig.Body[0] != interface{}(value) {
			t.Errorf("signal value mismatch: %d != %d", value, sig.Body[0])
		}
	}

	checkSignal(fetchSignal(t, ch, 50*time.Millisecond), 1)
	checkSignal(fetchSignal(t, ch, 50*time.Millisecond), 2)
	checkSignal(fetchSignal(t, ch, 50*time.Millisecond), 5)

	obj.RemoveMatchSignal(iface, "Heartbeat", WithMatchObjectPath(obj.Path()))
	select {
	case sig := <-ch:
		t.Errorf("Got signal after removing subscription: %v", sig)
	case <-time.After(200 * time.Millisecond):
	}
}
