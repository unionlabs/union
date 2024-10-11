package main

import (
	"fmt"
	"net"
	"os"

	"github.com/godbus/dbus"
)

// In order to enable TCP connections add the following configuration
// file to /etc/dbus-1/system.d or /etc/dbus-1/session.d, the location
// depends on your OS (you may update your /etc/dbus-1/session.conf instead):
//
// <!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
//  "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
// <busconfig>
//     <listen>tcp:host=localhost,bind=*,port=12345,family=ipv4</listen>
//     <listen>unix:tmpdir=/tmp</listen>
//     <auth>ANONYMOUS</auth>
//     <allow_anonymous/>
// </busconfig>
//
// If you're using systemd you may also need to reconfigure dbus.socket,
// say put this into /etc/systemd/system/dbus.socket.d/overrides.conf:
//
// [Socket]
// ListenStream=/var/run/dbus/system_bus_socket
// ListenStream=12345
//
// Reboot.

func main() {
	if len(os.Args) != 2 {
		fmt.Fprintf(os.Stderr, "usage: %s DBUS_TCP_ADDRESS\n", os.Args[0])
		os.Exit(1)
	}
	if err := run(os.Args[1]); err != nil {
		fmt.Fprintf(os.Stderr, "error: %s\n", err)
		os.Exit(2)
	}
	fmt.Println("ok")
}

func run(addr string) error {
	host, port, err := net.SplitHostPort(addr)
	if err != nil {
		return err
	}
	conn, err := dbus.Dial("tcp:host=" + host + ",port=" + port)
	if err != nil {
		return err
	}
	if err = conn.Auth([]dbus.Auth{dbus.AuthAnonymous()}); err != nil {
		return err
	}
	return conn.Hello()
}
