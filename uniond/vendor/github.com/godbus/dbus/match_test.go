package dbus

import "testing"

func TestFormatMatchOptions(t *testing.T) {
	opts := []MatchOption{
		withMatchType("signal"),
		WithMatchSender("org.bluez"),
		WithMatchInterface("org.freedesktop.DBus.Properties"),
		WithMatchMember("PropertiesChanged"),
		WithMatchPathNamespace("/org/bluez/hci0"),
	}
	want := "type='signal',sender='org.bluez'," +
		"interface='org.freedesktop.DBus.Properties'," +
		"member='PropertiesChanged',path_namespace='/org/bluez/hci0'"
	if have := formatMatchOptions(opts); have != want {
		t.Fatalf("formatMatchOptions(%v) = %q, want %q", opts, have, want)
	}
}
