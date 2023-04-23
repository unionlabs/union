package libsecret

import "github.com/godbus/dbus"


type Item struct {
  conn *dbus.Conn
  dbus  dbus.BusObject
}


func NewItem(conn *dbus.Conn, path dbus.ObjectPath) *Item {
  return &Item{
    conn: conn,
    dbus: conn.Object(DBusServiceName, path),
  }
}


func (item Item) Path() dbus.ObjectPath {
  return item.dbus.Path()
}


// READWRITE String Label;
func (item *Item) Label() (string, error) {
  val, err := item.dbus.GetProperty("org.freedesktop.Secret.Item.Label")
  if err != nil {
    return "", err
  }

  return val.Value().(string), nil
}


// READ Boolean Locked;
func (item *Item) Locked() (bool, error) {
  val, err := item.dbus.GetProperty("org.freedesktop.Secret.Item.Locked")
  if err != nil {
    return true, err
  }

  return val.Value().(bool), nil
}


// GetSecret (IN ObjectPath session, OUT Secret secret);
func (item *Item) GetSecret(session *Session) (*Secret, error) {
  secret := Secret{}

  err := item.dbus.Call("org.freedesktop.Secret.Item.GetSecret", 0, session.Path()).Store(&secret)
  if err != nil {
    return &Secret{}, err
  }

  return &secret, nil
}


// Delete (OUT ObjectPath Prompt);
func (item *Item) Delete() error {
  var prompt dbus.ObjectPath

  err := item.dbus.Call("org.freedesktop.Secret.Item.Delete", 0).Store(&prompt)
  if err != nil {
    return err
  }

  if isPrompt(prompt) {
    prompt := NewPrompt(item.conn, prompt)
    if _, err := prompt.Prompt(); err != nil {
      return err
    }
  }

  return nil
}
