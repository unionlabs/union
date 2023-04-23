package libsecret

import "github.com/godbus/dbus"


type Secret struct {
  Session        dbus.ObjectPath
  Parameters   []byte
  Value        []byte
  ContentType    string
}


func NewSecret(session *Session, params []byte, value []byte, contentType string) *Secret {
  return &Secret{
    Session:     session.Path(),
    Parameters:  params,
    Value:       value,
    ContentType: contentType,
  }
}
