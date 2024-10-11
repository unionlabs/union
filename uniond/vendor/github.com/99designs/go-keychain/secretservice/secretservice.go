package secretservice

import (
	"math/big"
	"time"

	dbus "github.com/keybase/go.dbus"
	errors "github.com/pkg/errors"
)

const SecretServiceInterface = "org.freedesktop.secrets"
const SecretServiceObjectPath dbus.ObjectPath = "/org/freedesktop/secrets"

// DefaultCollection need not necessarily exist in the user's keyring.
const DefaultCollection dbus.ObjectPath = "/org/freedesktop/secrets/aliases/default"

type authenticationMode string

const AuthenticationInsecurePlain authenticationMode = "plain"
const AuthenticationDHAES authenticationMode = "dh-ietf1024-sha256-aes128-cbc-pkcs7"

const NilFlags = 0

type Attributes map[string]string

type Secret struct {
	Session     dbus.ObjectPath
	Parameters  []byte
	Value       []byte
	ContentType string
}

type PromptCompletedResult struct {
	Dismissed bool
	Paths     dbus.Variant
}

type SecretService struct {
	conn               *dbus.Conn
	signalCh           <-chan *dbus.Signal
	sessionOpenTimeout time.Duration
}

type Session struct {
	Mode    authenticationMode
	Path    dbus.ObjectPath
	Public  *big.Int
	Private *big.Int
	AESKey  []byte
}

const DefaultSessionOpenTimeout = 10 * time.Second

func NewService() (*SecretService, error) {
	conn, err := dbus.SessionBus()
	if err != nil {
		return nil, errors.Wrap(err, "failed to open dbus connection")
	}
	signalCh := make(chan *dbus.Signal, 16)
	conn.Signal(signalCh)
	return &SecretService{conn: conn, signalCh: signalCh, sessionOpenTimeout: DefaultSessionOpenTimeout}, nil
}

func (s *SecretService) SetSessionOpenTimeout(d time.Duration) {
	s.sessionOpenTimeout = d
}

func (s *SecretService) ServiceObj() *dbus.Object {
	return s.conn.Object(SecretServiceInterface, SecretServiceObjectPath)
}

func (s *SecretService) Obj(path dbus.ObjectPath) *dbus.Object {
	return s.conn.Object(SecretServiceInterface, path)
}

type sessionOpenResponse struct {
	algorithmOutput dbus.Variant
	path            dbus.ObjectPath
}

func (s *SecretService) openSessionRaw(mode authenticationMode, sessionAlgorithmInput dbus.Variant) (resp sessionOpenResponse, err error) {
	err = s.ServiceObj().
		Call("org.freedesktop.Secret.Service.OpenSession", NilFlags, mode, sessionAlgorithmInput).
		Store(&resp.algorithmOutput, &resp.path)
	return resp, errors.Wrap(err, "failed to open secretservice session")
}

func (s *SecretService) OpenSession(mode authenticationMode) (session *Session, err error) {
	var sessionAlgorithmInput dbus.Variant

	session = new(Session)

	session.Mode = mode

	switch mode {
	case AuthenticationInsecurePlain:
		sessionAlgorithmInput = dbus.MakeVariant("")
	case AuthenticationDHAES:
		group := rfc2409SecondOakleyGroup()
		private, public, err := group.NewKeypair()
		if err != nil {
			return nil, err
		}
		session.Private = private
		session.Public = public
		sessionAlgorithmInput = dbus.MakeVariant(public.Bytes()) // math/big.Int.Bytes is big endian
	default:
		return nil, errors.Errorf("unknown authentication mode %v", mode)
	}

	sessionOpenCh := make(chan sessionOpenResponse)
	errCh := make(chan error)
	go func() {
		sessionOpenResponse, err := s.openSessionRaw(mode, sessionAlgorithmInput)
		if err != nil {
			errCh <- err
		} else {
			sessionOpenCh <- sessionOpenResponse
		}
	}()

	var sessionAlgorithmOutput dbus.Variant
	// NOTE: If the timeout case is reached, the above goroutine is leaked.
	// This is not terrible because D-Bus calls have an internal 2-mintue
	// timeout, so the goroutine will finish eventually. If two OpenSessions
	// are called at the saime time, they'll be on different channels so
	// they won't interfere with each other.
	select {
	case resp := <-sessionOpenCh:
		sessionAlgorithmOutput = resp.algorithmOutput
		session.Path = resp.path
	case err := <-errCh:
		return nil, err
	case <-time.After(s.sessionOpenTimeout):
		return nil, errors.Errorf("timed out after %s", s.sessionOpenTimeout)
	}

	switch mode {
	case AuthenticationInsecurePlain:
	case AuthenticationDHAES:
		theirPublicBigEndian, ok := sessionAlgorithmOutput.Value().([]byte)
		if !ok {
			return nil, errors.Errorf("failed to coerce algorithm output value to byteslice")
		}
		group := rfc2409SecondOakleyGroup()
		theirPublic := new(big.Int)
		theirPublic.SetBytes(theirPublicBigEndian)
		aesKey, err := group.keygenHKDFSHA256AES128(theirPublic, session.Private)
		if err != nil {
			return nil, err
		}
		session.AESKey = aesKey
	default:
		return nil, errors.Errorf("unknown authentication mode %v", mode)
	}

	return session, nil
}

func (s *SecretService) CloseSession(session *Session) {
	s.Obj(session.Path).Call("org.freedesktop.Secret.Session.Close", NilFlags)
}

func (s *SecretService) SearchCollection(collection dbus.ObjectPath, attributes Attributes) (items []dbus.ObjectPath, err error) {
	err = s.Obj(collection).
		Call("org.freedesktop.Secret.Collection.SearchItems", NilFlags, attributes).
		Store(&items)
	if err != nil {
		return nil, errors.Wrap(err, "failed to search collection")
	}
	return items, nil
}

type replaceBehavior int

const ReplaceBehaviorDoNotReplace = 0
const ReplaceBehaviorReplace = 1

func (s *SecretService) CreateItem(collection dbus.ObjectPath, properties map[string]dbus.Variant, secret Secret, replaceBehavior replaceBehavior) (item dbus.ObjectPath, err error) {
	var replace bool
	switch replaceBehavior {
	case ReplaceBehaviorDoNotReplace:
		replace = false
	case ReplaceBehaviorReplace:
		replace = true
	default:
		return "", errors.Errorf("unknown replace behavior %v", replaceBehavior)
	}

	var prompt dbus.ObjectPath
	err = s.Obj(collection).
		Call("org.freedesktop.Secret.Collection.CreateItem", NilFlags, properties, secret, replace).
		Store(&item, &prompt)
	if err != nil {
		return "", errors.Wrap(err, "failed to create item")
	}
	_, err = s.PromptAndWait(prompt)
	if err != nil {
		return "", err
	}
	return item, nil
}

func (s *SecretService) DeleteItem(item dbus.ObjectPath) (err error) {
	var prompt dbus.ObjectPath
	err = s.Obj(item).
		Call("org.freedesktop.Secret.Item.Delete", NilFlags).
		Store(&prompt)
	if err != nil {
		return errors.Wrap(err, "failed to delete item")
	}
	_, err = s.PromptAndWait(prompt)
	if err != nil {
		return err
	}
	return nil
}

func (s *SecretService) GetAttributes(item dbus.ObjectPath) (attributes Attributes, err error) {
	attributesV, err := s.Obj(item).GetProperty("org.freedesktop.Secret.Item.Attributes")
	if err != nil {
		return nil, errors.Wrap(err, "failed to get attributes")
	}
	attributesMap, ok := attributesV.Value().(map[string]string)
	if !ok {
		return nil, errors.Errorf("failed to coerce item attributes")
	}
	return Attributes(attributesMap), nil
}

func (s *SecretService) GetSecret(item dbus.ObjectPath, session Session) (secretPlaintext []byte, err error) {
	var secretI []interface{}
	err = s.Obj(item).
		Call("org.freedesktop.Secret.Item.GetSecret", NilFlags, session.Path).
		Store(&secretI)
	if err != nil {
		return nil, errors.Wrap(err, "failed to get secret")
	}
	secret := new(Secret)
	err = dbus.Store(secretI, &secret.Session, &secret.Parameters, &secret.Value, &secret.ContentType)
	if err != nil {
		return nil, errors.Wrap(err, "failed to unmarshal get secret result")
	}

	switch session.Mode {
	case AuthenticationInsecurePlain:
		secretPlaintext = secret.Value
	case AuthenticationDHAES:
		plaintext, err := unauthenticatedAESCBCDecrypt(secret.Parameters, secret.Value, session.AESKey)
		if err != nil {
			return nil, nil
		}
		secretPlaintext = plaintext
	default:
		return nil, errors.Errorf("cannot make secret for authentication mode %v", session.Mode)
	}

	return secretPlaintext, nil
}

const NullPrompt = "/"

func (s *SecretService) Unlock(items []dbus.ObjectPath) (err error) {
	var dummy []dbus.ObjectPath
	var prompt dbus.ObjectPath
	err = s.ServiceObj().
		Call("org.freedesktop.Secret.Service.Unlock", NilFlags, items).
		Store(&dummy, &prompt)
	if err != nil {
		return errors.Wrap(err, "failed to unlock items")
	}
	_, err = s.PromptAndWait(prompt)
	if err != nil {
		return errors.Wrap(err, "failed to prompt")
	}
	return nil
}

func (s *SecretService) LockItems(items []dbus.ObjectPath) (err error) {
	var dummy []dbus.ObjectPath
	var prompt dbus.ObjectPath
	err = s.ServiceObj().
		Call("org.freedesktop.Secret.Service.Lock", NilFlags, items).
		Store(&dummy, &prompt)
	if err != nil {
		return errors.Wrap(err, "failed to lock items")
	}
	_, err = s.PromptAndWait(prompt)
	if err != nil {
		return errors.Wrap(err, "failed to prompt")
	}
	return nil
}

type PromptDismissedError struct {
	err error
}

func (p PromptDismissedError) Error() string {
	return p.err.Error()
}

// PromptAndWait is NOT thread-safe.
func (s *SecretService) PromptAndWait(prompt dbus.ObjectPath) (paths *dbus.Variant, err error) {
	if prompt == NullPrompt {
		return nil, nil
	}
	call := s.Obj(prompt).Call("org.freedesktop.Secret.Prompt.Prompt", NilFlags, "Keyring Prompt")
	if call.Err != nil {
		return nil, errors.Wrap(err, "failed to prompt")
	}
	for {
		var result PromptCompletedResult
		select {
		case signal := <-s.signalCh:
			if signal.Name != "org.freedesktop.Secret.Prompt.Completed" {
				continue
			}
			err = dbus.Store(signal.Body, &result.Dismissed, &result.Paths)
			if err != nil {
				return nil, errors.Wrap(err, "failed to unmarshal prompt result")
			}
			if result.Dismissed {
				return nil, PromptDismissedError{errors.New("prompt dismissed")}
			}
			return &result.Paths, nil
		case <-time.After(30 * time.Second):
			return nil, errors.New("prompt timed out")
		}
	}
}

func NewSecretProperties(label string, attributes map[string]string) map[string]dbus.Variant {
	return map[string]dbus.Variant{
		"org.freedesktop.Secret.Item.Label":      dbus.MakeVariant(label),
		"org.freedesktop.Secret.Item.Attributes": dbus.MakeVariant(attributes),
	}
}

func (session *Session) NewSecret(secretBytes []byte) (Secret, error) {
	switch session.Mode {
	case AuthenticationInsecurePlain:
		return Secret{
			Session:     session.Path,
			Parameters:  nil,
			Value:       secretBytes,
			ContentType: "application/octet-stream",
		}, nil
	case AuthenticationDHAES:
		iv, ciphertext, err := unauthenticatedAESCBCEncrypt(secretBytes, session.AESKey)
		if err != nil {
			return Secret{}, err
		}
		return Secret{
			Session:     session.Path,
			Parameters:  iv,
			Value:       ciphertext,
			ContentType: "application/octet-stream",
		}, nil
	default:
		return Secret{}, errors.Errorf("cannot make secret for authentication mode %v", session.Mode)
	}
}
