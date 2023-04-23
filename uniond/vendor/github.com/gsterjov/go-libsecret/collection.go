package libsecret

import "github.com/godbus/dbus"


type Collection struct {
  conn *dbus.Conn
  dbus  dbus.BusObject
}


func NewCollection(conn *dbus.Conn, path dbus.ObjectPath) *Collection {
  return &Collection{
    conn: conn,
    dbus: conn.Object(DBusServiceName, path),
  }
}


func (collection Collection) Path() dbus.ObjectPath {
  return collection.dbus.Path()
}


// READ Array<ObjectPath> Items;
func (collection *Collection) Items() ([]Item, error) {
  val, err := collection.dbus.GetProperty("org.freedesktop.Secret.Collection.Items")
  if err != nil {
    return []Item{}, err
  }

  items := []Item{}
  for _, path := range val.Value().([]dbus.ObjectPath) {
    items = append(items, *NewItem(collection.conn, path))
  }

  return items, nil
}


// Delete (OUT ObjectPath prompt);
func (collection *Collection) Delete() error {
  var prompt dbus.ObjectPath

  err := collection.dbus.Call("org.freedesktop.Secret.Collection.Delete", 0).Store(&prompt)
  if err != nil {
    return err
  }

  if isPrompt(prompt) {
    prompt := NewPrompt(collection.conn, prompt)

    _, err := prompt.Prompt()
    if err != nil {
      return err
    }
  }

  return nil
}


// SearchItems (IN Dict<String,String> attributes, OUT Array<ObjectPath> results);
func (collection *Collection) SearchItems(profile string) ([]Item, error) {
  attributes := make(map[string]string)
  attributes["profile"] = profile

  var paths []dbus.ObjectPath

  err := collection.dbus.Call("org.freedesktop.Secret.Collection.SearchItems", 0, attributes).Store(&paths)
  if err != nil {
    return []Item{}, err
  }

  items := []Item{}
  for _, path := range paths {
    items = append(items, *NewItem(collection.conn, path))
  }

  return items, nil
}


// CreateItem (IN Dict<String,Variant> properties, IN Secret secret, IN Boolean replace, OUT ObjectPath item, OUT ObjectPath prompt);
func (collection *Collection) CreateItem(label string, secret *Secret, replace bool) (*Item, error) {
  properties := make(map[string]dbus.Variant)
  attributes := make(map[string]string)

  attributes["profile"] = label
  properties["org.freedesktop.Secret.Item.Label"] = dbus.MakeVariant(label)
  properties["org.freedesktop.Secret.Item.Attributes"] = dbus.MakeVariant(attributes)

  var path   dbus.ObjectPath
  var prompt dbus.ObjectPath

  err := collection.dbus.Call("org.freedesktop.Secret.Collection.CreateItem", 0, properties, secret, replace).Store(&path, &prompt)
  if err != nil {
    return &Item{}, err
  }

  if isPrompt(prompt) {
    prompt := NewPrompt(collection.conn, prompt)

    result, err := prompt.Prompt()
    if err != nil {
      return &Item{}, err
    }

    path = result.Value().(dbus.ObjectPath)
  }

  return NewItem(collection.conn, path), nil
}


// READ Boolean Locked;
func (collection *Collection) Locked() (bool, error) {
  val, err := collection.dbus.GetProperty("org.freedesktop.Secret.Collection.Locked")
  if err != nil {
    return true, err
  }

  return val.Value().(bool), nil
}
