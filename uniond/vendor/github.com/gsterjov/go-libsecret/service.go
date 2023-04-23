package libsecret

import "github.com/godbus/dbus"


const (
  DBusServiceName = "org.freedesktop.secrets"
  DBusPath = "/org/freedesktop/secrets"
)

type DBusObject interface {
  Path() dbus.ObjectPath
}


type Service struct {
  conn *dbus.Conn
  dbus  dbus.BusObject
}


func NewService() (*Service, error) {
  conn, err := dbus.SessionBus()
  if err != nil {
    return &Service{}, err
  }

  return &Service{
    conn: conn,
    dbus: conn.Object(DBusServiceName, DBusPath),
  }, nil
}


func (service Service) Path() dbus.ObjectPath {
  return service.dbus.Path()
}


// OpenSession (IN String algorithm, IN Variant input, OUT Variant output, OUT ObjectPath result);
func (service *Service) Open() (*Session, error) {
  var output dbus.Variant
  var path   dbus.ObjectPath

  err := service.dbus.Call("org.freedesktop.Secret.Service.OpenSession", 0, "plain", dbus.MakeVariant("")).Store(&output, &path)
  if err != nil {
    return &Session{}, err
  }

  return NewSession(service.conn, path), nil
}


// READ Array<ObjectPath> Collections;
func (service *Service) Collections() ([]Collection, error) {
  val, err := service.dbus.GetProperty("org.freedesktop.Secret.Service.Collections")
  if err != nil {
    return []Collection{}, err
  }

  collections := []Collection{}
  for _, path := range val.Value().([]dbus.ObjectPath) {
    collections = append(collections, *NewCollection(service.conn, path))
  }

  return collections, nil
}


// CreateCollection (IN Dict<String,Variant> properties, IN String alias, OUT ObjectPath collection, OUT ObjectPath prompt);
func (service *Service) CreateCollection(label string) (*Collection, error) {
  properties := make(map[string]dbus.Variant)
  properties["org.freedesktop.Secret.Collection.Label"] = dbus.MakeVariant(label)

  var path   dbus.ObjectPath
  var prompt dbus.ObjectPath

  err := service.dbus.Call("org.freedesktop.Secret.Service.CreateCollection", 0, properties, "").Store(&path, &prompt)
  if err != nil {
    return &Collection{}, err
  }

  if isPrompt(prompt) {
    prompt := NewPrompt(service.conn, prompt)

    result, err := prompt.Prompt()
    if err != nil {
      return &Collection{}, err
    }

    path = result.Value().(dbus.ObjectPath)
  }

  return NewCollection(service.conn, path), nil
}


// Unlock (IN Array<ObjectPath> objects, OUT Array<ObjectPath> unlocked, OUT ObjectPath prompt);
func (service *Service) Unlock(object DBusObject) error {
  objects := []dbus.ObjectPath{object.Path()}

  var unlocked []dbus.ObjectPath
  var prompt     dbus.ObjectPath

  err := service.dbus.Call("org.freedesktop.Secret.Service.Unlock", 0, objects).Store(&unlocked, &prompt)
  if err != nil {
    return err
  }

  if isPrompt(prompt) {
    prompt := NewPrompt(service.conn, prompt)
    if _, err := prompt.Prompt(); err != nil {
      return err
    }
  }

  return nil
}


// Lock (IN Array<ObjectPath> objects, OUT Array<ObjectPath> locked, OUT ObjectPath Prompt);
func (service *Service) Lock(object DBusObject) error {
  objects := []dbus.ObjectPath{object.Path()}

  var locked []dbus.ObjectPath
  var prompt   dbus.ObjectPath

  err := service.dbus.Call("org.freedesktop.Secret.Service.Lock", 0, objects).Store(&locked, &prompt)
  if err != nil {
    return err
  }

  if isPrompt(prompt) {
    prompt := NewPrompt(service.conn, prompt)
    if _, err := prompt.Prompt(); err != nil {
      return err
    }
  }

  return nil
}
