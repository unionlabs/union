package zk

import (
	"errors"
	"net"
	"strings"
	"time"

	"github.com/go-zookeeper/zk"

	"github.com/go-kit/log"
)

// DefaultACL is the default ACL to use for creating znodes.
var (
	DefaultACL            = zk.WorldACL(zk.PermAll)
	ErrInvalidCredentials = errors.New("invalid credentials provided")
	ErrClientClosed       = errors.New("client service closed")
	ErrNotRegistered      = errors.New("not registered")
	ErrNodeNotFound       = errors.New("node not found")
)

const (
	// DefaultConnectTimeout is the default timeout to establish a connection to
	// a ZooKeeper node.
	DefaultConnectTimeout = 2 * time.Second
	// DefaultSessionTimeout is the default timeout to keep the current
	// ZooKeeper session alive during a temporary disconnect.
	DefaultSessionTimeout = 5 * time.Second
)

// Client is a wrapper around a lower level ZooKeeper client implementation.
type Client interface {
	// GetEntries should query the provided path in ZooKeeper, place a watch on
	// it and retrieve data from its current child nodes.
	GetEntries(path string) ([]string, <-chan zk.Event, error)
	// CreateParentNodes should try to create the path in case it does not exist
	// yet on ZooKeeper.
	CreateParentNodes(path string) error
	// Register a service with ZooKeeper.
	Register(s *Service) error
	// Deregister a service with ZooKeeper.
	Deregister(s *Service) error
	// Stop should properly shutdown the client implementation
	Stop()
}

type clientConfig struct {
	logger          log.Logger
	acl             []zk.ACL
	credentials     []byte
	connectTimeout  time.Duration
	sessionTimeout  time.Duration
	rootNodePayload [][]byte
	eventHandler    func(zk.Event)
}

// Option functions enable friendly APIs.
type Option func(*clientConfig) error

type client struct {
	*zk.Conn
	clientConfig
	active bool
	quit   chan struct{}
}

// ACL returns an Option specifying a non-default ACL for creating parent nodes.
func ACL(acl []zk.ACL) Option {
	return func(c *clientConfig) error {
		c.acl = acl
		return nil
	}
}

// Credentials returns an Option specifying a user/password combination which
// the client will use to authenticate itself with.
func Credentials(user, pass string) Option {
	return func(c *clientConfig) error {
		if user == "" || pass == "" {
			return ErrInvalidCredentials
		}
		c.credentials = []byte(user + ":" + pass)
		return nil
	}
}

// ConnectTimeout returns an Option specifying a non-default connection timeout
// when we try to establish a connection to a ZooKeeper server.
func ConnectTimeout(t time.Duration) Option {
	return func(c *clientConfig) error {
		if t.Seconds() < 1 {
			return errors.New("invalid connect timeout (minimum value is 1 second)")
		}
		c.connectTimeout = t
		return nil
	}
}

// SessionTimeout returns an Option specifying a non-default session timeout.
func SessionTimeout(t time.Duration) Option {
	return func(c *clientConfig) error {
		if t.Seconds() < 1 {
			return errors.New("invalid session timeout (minimum value is 1 second)")
		}
		c.sessionTimeout = t
		return nil
	}
}

// Payload returns an Option specifying non-default data values for each znode
// created by CreateParentNodes.
func Payload(payload [][]byte) Option {
	return func(c *clientConfig) error {
		c.rootNodePayload = payload
		return nil
	}
}

// EventHandler returns an Option specifying a callback function to handle
// incoming zk.Event payloads (ZooKeeper connection events).
func EventHandler(handler func(zk.Event)) Option {
	return func(c *clientConfig) error {
		c.eventHandler = handler
		return nil
	}
}

// NewClient returns a ZooKeeper client with a connection to the server cluster.
// It will return an error if the server cluster cannot be resolved.
func NewClient(servers []string, logger log.Logger, options ...Option) (Client, error) {
	defaultEventHandler := func(event zk.Event) {
		logger.Log("eventtype", event.Type.String(), "server", event.Server, "state", event.State.String(), "err", event.Err)
	}
	config := clientConfig{
		acl:            DefaultACL,
		connectTimeout: DefaultConnectTimeout,
		sessionTimeout: DefaultSessionTimeout,
		eventHandler:   defaultEventHandler,
		logger:         logger,
	}
	for _, option := range options {
		if err := option(&config); err != nil {
			return nil, err
		}
	}
	// dialer overrides the default ZooKeeper library Dialer so we can configure
	// the connectTimeout. The current library has a hardcoded value of 1 second
	// and there are reports of race conditions, due to slow DNS resolvers and
	// other network latency issues.
	dialer := func(network, address string, _ time.Duration) (net.Conn, error) {
		return net.DialTimeout(network, address, config.connectTimeout)
	}
	conn, eventc, err := zk.Connect(servers, config.sessionTimeout, withLogger(logger), zk.WithDialer(dialer))

	if err != nil {
		return nil, err
	}

	if len(config.credentials) > 0 {
		err = conn.AddAuth("digest", config.credentials)
		if err != nil {
			return nil, err
		}
	}

	c := &client{conn, config, true, make(chan struct{})}

	// Start listening for incoming Event payloads and callback the set
	// eventHandler.
	go func() {
		for {
			select {
			case event := <-eventc:
				config.eventHandler(event)
			case <-c.quit:
				return
			}
		}
	}()
	return c, nil
}

// CreateParentNodes implements the ZooKeeper Client interface.
func (c *client) CreateParentNodes(path string) error {
	if !c.active {
		return ErrClientClosed
	}
	if path[0] != '/' {
		return zk.ErrInvalidPath
	}
	payload := []byte("")
	pathString := ""
	pathNodes := strings.Split(path, "/")
	for i := 1; i < len(pathNodes); i++ {
		if i <= len(c.rootNodePayload) {
			payload = c.rootNodePayload[i-1]
		} else {
			payload = []byte("")
		}
		pathString += "/" + pathNodes[i]
		_, err := c.Create(pathString, payload, 0, c.acl)
		// not being able to create the node because it exists or not having
		// sufficient rights is not an issue. It is ok for the node to already
		// exist and/or us to only have read rights
		if err != nil && err != zk.ErrNodeExists && err != zk.ErrNoAuth {
			return err
		}
	}
	return nil
}

// GetEntries implements the ZooKeeper Client interface.
func (c *client) GetEntries(path string) ([]string, <-chan zk.Event, error) {
	// retrieve list of child nodes for given path and add watch to path
	znodes, _, eventc, err := c.ChildrenW(path)

	if err != nil {
		return nil, eventc, err
	}

	var resp []string
	for _, znode := range znodes {
		// retrieve payload for child znode and add to response array
		if data, _, err := c.Get(path + "/" + znode); err == nil {
			resp = append(resp, string(data))
		}
	}
	return resp, eventc, nil
}

// Register implements the ZooKeeper Client interface.
func (c *client) Register(s *Service) error {
	if s.Path[len(s.Path)-1] != '/' {
		s.Path += "/"
	}
	path := s.Path + s.Name
	if err := c.CreateParentNodes(path); err != nil {
		return err
	}
	if path[len(path)-1] != '/' {
		path += "/"
	}
	node, err := c.CreateProtectedEphemeralSequential(path, s.Data, c.acl)
	if err != nil {
		return err
	}
	s.node = node
	return nil
}

// Deregister implements the ZooKeeper Client interface.
func (c *client) Deregister(s *Service) error {
	if s.node == "" {
		return ErrNotRegistered
	}
	path := s.Path + s.Name
	found, stat, err := c.Exists(path)
	if err != nil {
		return err
	}
	if !found {
		return ErrNodeNotFound
	}
	if err := c.Delete(path, stat.Version); err != nil {
		return err
	}
	return nil
}

// Stop implements the ZooKeeper Client interface.
func (c *client) Stop() {
	c.active = false
	close(c.quit)
	c.Close()
}
