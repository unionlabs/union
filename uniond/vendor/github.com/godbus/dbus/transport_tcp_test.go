package dbus

import (
	"fmt"
	"net"
	"testing"
)

func TestTcpConnection(t *testing.T) {
	listener, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatal("Failed to create listener")
	}
	host, port, err := net.SplitHostPort(listener.Addr().String())
	if err != nil {
		t.Fatal("Failed to parse host/port")
	}

	conn, err := Dial(fmt.Sprintf("tcp:host=%s,port=%s", host, port))
	if err != nil {
		t.Error("Expected no error, got", err)
	}
	if conn == nil {
		t.Error("Expected connection, got nil")
	}
}
