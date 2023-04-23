package libsecret

import "github.com/godbus/dbus"


type Session struct {
  conn *dbus.Conn
  dbus  dbus.BusObject
}


func NewSession(conn *dbus.Conn, path dbus.ObjectPath) *Session {
  return &Session{
    conn: conn,
    dbus: conn.Object(DBusServiceName, path),
  }
}


func (session Session) Path() dbus.ObjectPath {
  return session.dbus.Path()
}
