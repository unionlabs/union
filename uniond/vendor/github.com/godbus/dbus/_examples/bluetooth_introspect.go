package main

import (
	"fmt"
	"os"

	"github.com/godbus/dbus"
)

func main() {
	conn, err := dbus.SystemBus()
	if err != nil {
		fmt.Fprintln(os.Stderr, "Failed to connect to SystemBus bus:", err)
		os.Exit(1)
	}

	var s string
	err = conn.Object("org.bluez", "/").Call("org.freedesktop.DBus.Introspectable.Introspect", 0).Store(&s)
	if err != nil {
		fmt.Fprintln(os.Stderr, "Failed to introspect bluez", err)
		os.Exit(1)
	}

	fmt.Println(s)
}
