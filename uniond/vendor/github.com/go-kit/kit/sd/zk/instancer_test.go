package zk

import (
	"testing"
	"time"

	"github.com/go-kit/kit/sd"
)

var _ sd.Instancer = (*Instancer)(nil) // API check

func TestInstancer(t *testing.T) {
	client := newFakeClient()

	instancer, err := NewInstancer(client, path, logger)
	if err != nil {
		t.Fatalf("failed to create new Instancer: %v", err)
	}
	defer instancer.Stop()
	endpointer := sd.NewEndpointer(instancer, newFactory(""), logger)

	if _, err := endpointer.Endpoints(); err != nil {
		t.Fatal(err)
	}
}

func TestBadFactory(t *testing.T) {
	client := newFakeClient()

	instancer, err := NewInstancer(client, path, logger)
	if err != nil {
		t.Fatalf("failed to create new Instancer: %v", err)
	}
	defer instancer.Stop()
	endpointer := sd.NewEndpointer(instancer, newFactory("kaboom"), logger)

	// instance1 came online
	client.AddService(path+"/instance1", "kaboom")

	// instance2 came online
	client.AddService(path+"/instance2", "zookeeper_node_data")

	if err = asyncTest(100*time.Millisecond, 1, endpointer); err != nil {
		t.Error(err)
	}
}

func TestServiceUpdate(t *testing.T) {
	client := newFakeClient()

	instancer, err := NewInstancer(client, path, logger)
	if err != nil {
		t.Fatalf("failed to create new Instancer: %v", err)
	}
	defer instancer.Stop()
	endpointer := sd.NewEndpointer(instancer, newFactory(""), logger)

	endpoints, err := endpointer.Endpoints()
	if err != nil {
		t.Fatal(err)
	}
	if want, have := 0, len(endpoints); want != have {
		t.Errorf("want %d, have %d", want, have)
	}

	// instance1 came online
	client.AddService(path+"/instance1", "zookeeper_node_data1")

	// instance2 came online
	client.AddService(path+"/instance2", "zookeeper_node_data2")

	// we should have 2 instances
	if err = asyncTest(100*time.Millisecond, 2, endpointer); err != nil {
		t.Error(err)
	}

	// TODO(pb): this bit is flaky
	//
	//// watch triggers an error...
	//client.SendErrorOnWatch()
	//
	//// test if error was consumed
	//if err = client.ErrorIsConsumedWithin(100 * time.Millisecond); err != nil {
	//	t.Error(err)
	//}

	// instance3 came online
	client.AddService(path+"/instance3", "zookeeper_node_data3")

	// we should have 3 instances
	if err = asyncTest(100*time.Millisecond, 3, endpointer); err != nil {
		t.Error(err)
	}

	// instance1 goes offline
	client.RemoveService(path + "/instance1")

	// instance2 goes offline
	client.RemoveService(path + "/instance2")

	// we should have 1 instance
	if err = asyncTest(100*time.Millisecond, 1, endpointer); err != nil {
		t.Error(err)
	}
}

func TestBadInstancerCreate(t *testing.T) {
	client := newFakeClient()
	client.SendErrorOnWatch()

	instancer, err := NewInstancer(client, path, logger)
	if err == nil {
		t.Error("expected error on new Instancer")
	}
	if instancer != nil {
		t.Error("expected Instancer not to be created")
	}
	instancer, err = NewInstancer(client, "BadPath", logger)
	if err == nil {
		t.Error("expected error on new Instancer")
	}
	if instancer != nil {
		t.Error("expected Instancer not to be created")
	}
}
