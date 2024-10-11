package dbus

import (
	"fmt"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

type tester struct {
	conn *Conn
	sigs chan *Signal

	subSigsMu sync.Mutex
	subSigs   map[string]map[string]struct{}

	serial uint32
}

type intro struct {
	path ObjectPath
}

func (i *intro) introspectPath(path ObjectPath) string {
	switch path {
	case "/":
		return `<node><node name="com"></node></node>`
	case "/com":
		return `<node><node name="github"></node></node>`
	case "/com/github":
		return `<node><node name="godbus"></node></node>`
	case "/com/github/godbus":
		return `<node><node name="tester"></node></node>`
	}
	return ""
}

func (i *intro) LookupInterface(name string) (Interface, bool) {
	if name == "org.freedesktop.DBus.Introspectable" {
		return i, true
	}
	return nil, false
}

func (i *intro) LookupMethod(name string) (Method, bool) {
	if name == "Introspect" {
		return intro_fn(func() string {
			return i.introspectPath(i.path)
		}), true
	}
	return nil, false
}

func newIntro(path ObjectPath) *intro {
	return &intro{path}
}

//Handler
func (t *tester) LookupObject(path ObjectPath) (ServerObject, bool) {
	if path == "/com/github/godbus/tester" {
		return t, true
	}
	return newIntro(path), true
}

//ServerObject
func (t *tester) LookupInterface(name string) (Interface, bool) {
	switch name {
	case "com.github.godbus.dbus.Tester":
		return t, true
	case "org.freedesktop.DBus.Introspectable":
		return t, true
	}

	return nil, false
}

//Interface
func (t *tester) LookupMethod(name string) (Method, bool) {
	switch name {
	case "Test":
		return t, true
	case "Error":
		return terrfn(func(in string) error {
			return fmt.Errorf(in)
		}), true
	case "Introspect":
		return intro_fn(func() string {
			return `<node>
    <interface name="org.freedesktop.DBus.Introspectable.Introspect">
        <method name="Introspect">
            <arg name="out" type="i" direction="out">
        </method>
    </interface>
    <interface name="com.github.godbus.dbus.Tester">
        <method name="Test">
            <arg name="in" type="i" direction="in">
            <arg name="out" type="i" direction="out">
        </method>
        <signal name="sig1">
            <arg name="out" type="i" direction="out">
        </signal>
    </interface>
</node>`
		}), true
	}
	return nil, false
}

//Method
func (t *tester) Call(args ...interface{}) ([]interface{}, error) {
	return args, nil
}

func (t *tester) NumArguments() int {
	return 1
}

func (t *tester) NumReturns() int {
	return 1
}

func (t *tester) ArgumentValue(position int) interface{} {
	return ""
}

func (t *tester) ReturnValue(position int) interface{} {
	return ""
}

type terrfn func(in string) error

func (t terrfn) Call(args ...interface{}) ([]interface{}, error) {
	return nil, t(*args[0].(*string))
}

func (t terrfn) NumArguments() int {
	return 1
}

func (t terrfn) NumReturns() int {
	return 0
}

func (t terrfn) ArgumentValue(position int) interface{} {
	return ""
}

func (t terrfn) ReturnValue(position int) interface{} {
	return ""
}

//SignalHandler
func (t *tester) DeliverSignal(iface, name string, signal *Signal) {
	t.subSigsMu.Lock()
	intf, ok := t.subSigs[iface]
	t.subSigsMu.Unlock()
	if !ok {
		return
	}
	if _, ok := intf[name]; !ok {
		return
	}
	t.sigs <- signal
}

func (t *tester) AddSignal(iface, name string) error {
	t.subSigsMu.Lock()
	if i, ok := t.subSigs[iface]; ok {
		i[name] = struct{}{}
	} else {
		t.subSigs[iface] = make(map[string]struct{})
		t.subSigs[iface][name] = struct{}{}
	}
	t.subSigsMu.Unlock()
	return t.conn.AddMatchSignal(WithMatchInterface(iface), WithMatchMember(name))
}

func (t *tester) Close() {
	t.conn.Close()
	close(t.sigs)
}

func (t *tester) Name() string {
	return t.conn.Names()[0]
}

func (t *tester) GetSerial() uint32 {
	return atomic.AddUint32(&t.serial, 1)
}

func (t *tester) RetireSerial(serial uint32) {}

type intro_fn func() string

func (intro intro_fn) Call(args ...interface{}) ([]interface{}, error) {
	return []interface{}{intro()}, nil
}

func (_ intro_fn) NumArguments() int {
	return 0
}

func (_ intro_fn) NumReturns() int {
	return 1
}

func (_ intro_fn) ArgumentValue(position int) interface{} {
	return nil
}

func (_ intro_fn) ReturnValue(position int) interface{} {
	return ""
}

func newTester() (*tester, error) {
	tester := &tester{
		sigs:    make(chan *Signal),
		subSigs: make(map[string]map[string]struct{}),
	}
	conn, err := SessionBusPrivate(
		WithHandler(tester),
		WithSignalHandler(tester),
		WithSerialGenerator(tester),
	)
	if err != nil {
		return nil, err
	}
	err = conn.Auth(nil)
	if err != nil {
		conn.Close()
		return nil, err
	}
	err = conn.Hello()
	if err != nil {
		conn.Close()
		return nil, err
	}
	tester.conn = conn
	return tester, nil
}

func TestHandlerCall(t *testing.T) {
	tester, err := newTester()
	if err != nil {
		t.Errorf("Unexpected error: %s", err)
	}
	conn, err := SessionBus()
	if err != nil {
		t.Errorf("Unexpected error: %s", err)
	}
	obj := conn.Object(tester.Name(), "/com/github/godbus/tester")
	var out string
	in := "foo"
	err = obj.Call("com.github.godbus.dbus.Tester.Test", 0, in).Store(&out)
	if err != nil {
		t.Errorf("Unexpected error: %s", err)
	}
	if out != in {
		t.Errorf("Unexpected error: got %s, expected %s", out, in)
	}
	tester.Close()
}

func TestHandlerCallGenericError(t *testing.T) {
	tester, err := newTester()
	if err != nil {
		t.Errorf("Unexpected error: %s", err)
	}
	conn, err := SessionBus()
	if err != nil {
		t.Errorf("Unexpected error: %s", err)
	}
	obj := conn.Object(tester.Name(), "/com/github/godbus/tester")
	var out string
	in := "foo"
	err = obj.Call("com.github.godbus.dbus.Tester.Error", 0, in).Store(&out)
	if err != nil && err.(Error).Body[0].(string) != "foo" {
		t.Errorf("Unexpected error: %s", err)
	}

	tester.Close()
}

func TestHandlerCallNonExistent(t *testing.T) {
	tester, err := newTester()
	if err != nil {
		t.Errorf("Unexpected error: %s", err)
	}
	conn, err := SessionBus()
	if err != nil {
		t.Errorf("Unexpected error: %s", err)
	}
	obj := conn.Object(tester.Name(), "/com/github/godbus/tester/nonexist")
	var out string
	in := "foo"
	err = obj.Call("com.github.godbus.dbus.Tester.Test", 0, in).Store(&out)
	if err != nil {
		if err.Error() != "Object does not implement the interface" {
			t.Errorf("Unexpected error: %s", err)
		}
	}
	tester.Close()
}

func TestHandlerInvalidFunc(t *testing.T) {
	tester, err := newTester()
	if err != nil {
		t.Errorf("Unexpected error: %s", err)
	}
	conn, err := SessionBus()
	if err != nil {
		t.Errorf("Unexpected error: %s", err)
	}
	obj := conn.Object(tester.Name(), "/com/github/godbus/tester")
	var out string
	in := "foo"
	err = obj.Call("com.github.godbus.dbus.Tester.Notexist", 0, in).Store(&out)
	if err == nil {
		t.Errorf("didn't get expected error")
	}
	tester.Close()
}

func TestHandlerInvalidNumArg(t *testing.T) {
	tester, err := newTester()
	if err != nil {
		t.Errorf("Unexpected error: %s", err)
	}
	conn, err := SessionBus()
	if err != nil {
		t.Errorf("Unexpected error: %s", err)
	}
	obj := conn.Object(tester.Name(), "/com/github/godbus/tester")
	var out string
	err = obj.Call("com.github.godbus.dbus.Tester.Test", 0).Store(&out)
	if err == nil {
		t.Errorf("didn't get expected error")
	}
	tester.Close()
}

func TestHandlerInvalidArgType(t *testing.T) {
	tester, err := newTester()
	if err != nil {
		t.Errorf("Unexpected error: %s", err)
	}
	conn, err := SessionBus()
	if err != nil {
		t.Errorf("Unexpected error: %s", err)
	}
	obj := conn.Object(tester.Name(), "/com/github/godbus/tester")
	var out string
	err = obj.Call("com.github.godbus.dbus.Tester.Test", 0, 2.10).Store(&out)
	if err == nil {
		t.Errorf("didn't get expected error")
	}
	tester.Close()
}

func TestHandlerIntrospect(t *testing.T) {
	tester, err := newTester()
	if err != nil {
		t.Errorf("Unexpected error: %s", err)
	}
	conn, err := SessionBus()
	if err != nil {
		t.Errorf("Unexpected error: %s", err)
	}
	obj := conn.Object(tester.Name(), "/com/github/godbus/tester")
	var out string
	err = obj.Call("org.freedesktop.DBus.Introspectable.Introspect", 0).Store(&out)
	if err != nil {
		t.Errorf("Unexpected error: %s", err)
	}
	expected := `<node>
    <interface name="org.freedesktop.DBus.Introspectable.Introspect">
        <method name="Introspect">
            <arg name="out" type="i" direction="out">
        </method>
    </interface>
    <interface name="com.github.godbus.dbus.Tester">
        <method name="Test">
            <arg name="in" type="i" direction="in">
            <arg name="out" type="i" direction="out">
        </method>
        <signal name="sig1">
            <arg name="out" type="i" direction="out">
        </signal>
    </interface>
</node>`
	if out != expected {
		t.Errorf("didn't get expected return value, expected %s got %s", expected, out)
	}
	tester.Close()
}

func TestHandlerIntrospectPath(t *testing.T) {
	tester, err := newTester()
	if err != nil {
		t.Errorf("Unexpected error: %s", err)
	}
	conn, err := SessionBus()
	if err != nil {
		t.Errorf("Unexpected error: %s", err)
	}
	obj := conn.Object(tester.Name(), "/com/github/godbus")
	var out string
	err = obj.Call("org.freedesktop.DBus.Introspectable.Introspect", 0).Store(&out)
	if err != nil {
		t.Errorf("Unexpected error: %s", err)
	}
	expected := `<node><node name="tester"></node></node>`
	if out != expected {
		t.Errorf("didn't get expected return value, expected %s got %s", expected, out)
	}
	tester.Close()
}

func TestHandlerSignal(t *testing.T) {
	tester, err := newTester()
	if err != nil {
		t.Errorf("Unexpected error: %s", err)
	}
	conn, err := SessionBus()
	if err != nil {
		t.Errorf("Unexpected error: %s", err)
	}
	if err = tester.AddSignal("com.github.godbus.dbus.Tester", "sig1"); err != nil {
		t.Fatal(err)
	}
	if err = conn.Emit(
		"/com/github/godbus/tester",
		"com.github.godbus.dbus.Tester.sig1",
		"foo",
	); err != nil {
		t.Fatal(err)
	}
	select {
	case sig := <-tester.sigs:
		if sig.Body[0] != "foo" {
			t.Errorf("Unexpected signal got %s, expected %s", sig.Body[0], "foo")
		}
	case <-time.After(time.Second * 10): //overly generous timeout
		t.Errorf("Didn't receive a signal after 10 seconds")
	}
	tester.Close()
}

type X struct {
}

func (x *X) Method1() *Error {
	return nil
}

func TestRaceInExport(t *testing.T) {
	const (
		dbusPath      = "/org/example/godbus/test1"
		dbusInterface = "org.example.godbus.test1"
	)

	bus, err := SessionBus()
	if err != nil {
		t.Fatal(err)
	}

	var x X

	var wg sync.WaitGroup
	wg.Add(2)
	go func() {
		err = bus.Export(&x, dbusPath, dbusInterface)
		if err != nil {
			t.Fatal(err)
		}
		wg.Done()
	}()

	go func() {
		obj := bus.Object(bus.Names()[0], dbusPath)
		obj.Call(dbusInterface+".Method1", 0)
		wg.Done()
	}()
	wg.Wait()
}
