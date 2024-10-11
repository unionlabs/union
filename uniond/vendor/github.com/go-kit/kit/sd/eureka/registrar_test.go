package eureka

import (
	"testing"
	"time"
)

func TestRegistrar(t *testing.T) {
	connection := &testConnection{
		errHeartbeat: errTest,
	}

	registrar1 := NewRegistrar(connection, instanceTest1, loggerTest)
	registrar2 := NewRegistrar(connection, instanceTest2, loggerTest)

	// Not registered.
	registrar1.Deregister()
	if want, have := 0, len(connection.instances); want != have {
		t.Errorf("want %d, have %d", want, have)
	}

	// Register.
	registrar1.Register()
	if want, have := 1, len(connection.instances); want != have {
		t.Errorf("want %d, have %d", want, have)
	}

	registrar2.Register()
	if want, have := 2, len(connection.instances); want != have {
		t.Errorf("want %d, have %d", want, have)
	}

	// Deregister.
	registrar1.Deregister()
	if want, have := 1, len(connection.instances); want != have {
		t.Errorf("want %d, have %d", want, have)
	}

	// Already registered.
	registrar1.Register()
	if want, have := 2, len(connection.instances); want != have {
		t.Errorf("want %d, have %d", want, have)
	}
	registrar1.Register()
	if want, have := 2, len(connection.instances); want != have {
		t.Errorf("want %d, have %d", want, have)
	}

	// Wait for a heartbeat failure.
	time.Sleep(1010 * time.Millisecond)
	if want, have := 2, len(connection.instances); want != have {
		t.Errorf("want %d, have %d", want, have)
	}
	registrar1.Deregister()
	if want, have := 1, len(connection.instances); want != have {
		t.Errorf("want %d, have %d", want, have)
	}
}

func TestBadRegister(t *testing.T) {
	connection := &testConnection{
		errRegister: errTest,
	}

	registrar := NewRegistrar(connection, instanceTest1, loggerTest)
	registrar.Register()
	if want, have := 0, len(connection.instances); want != have {
		t.Errorf("want %d, have %d", want, have)
	}
}

func TestBadDeregister(t *testing.T) {
	connection := &testConnection{
		errDeregister: errTest,
	}

	registrar := NewRegistrar(connection, instanceTest1, loggerTest)
	registrar.Register()
	if want, have := 1, len(connection.instances); want != have {
		t.Errorf("want %d, have %d", want, have)
	}
	registrar.Deregister()
	if want, have := 1, len(connection.instances); want != have {
		t.Errorf("want %d, have %d", want, have)
	}
}

func TestExpiredInstance(t *testing.T) {
	connection := &testConnection{
		errHeartbeat: errNotFound,
	}

	registrar := NewRegistrar(connection, instanceTest1, loggerTest)
	registrar.Register()

	// Wait for a heartbeat failure.
	time.Sleep(1010 * time.Millisecond)

	if want, have := 1, len(connection.instances); want != have {
		t.Errorf("want %d, have %d", want, have)
	}
}
