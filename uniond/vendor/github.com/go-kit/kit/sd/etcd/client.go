package etcd

import (
	"context"
	"crypto/tls"
	"crypto/x509"
	"errors"
	"io/ioutil"
	"net"
	"net/http"
	"time"

	etcd "go.etcd.io/etcd/client/v2"
)

var (
	// ErrNoKey indicates a client method needs a key but receives none.
	ErrNoKey = errors.New("no key provided")

	// ErrNoValue indicates a client method needs a value but receives none.
	ErrNoValue = errors.New("no value provided")
)

// Client is a wrapper around the etcd client.
type Client interface {
	// GetEntries queries the given prefix in etcd and returns a slice
	// containing the values of all keys found, recursively, underneath that
	// prefix.
	GetEntries(prefix string) ([]string, error)

	// WatchPrefix watches the given prefix in etcd for changes. When a change
	// is detected, it will signal on the passed channel. Clients are expected
	// to call GetEntries to update themselves with the latest set of complete
	// values. WatchPrefix will always send an initial sentinel value on the
	// channel after establishing the watch, to ensure that clients always
	// receive the latest set of values. WatchPrefix will block until the
	// context passed to the NewClient constructor is terminated.
	WatchPrefix(prefix string, ch chan struct{})

	// Register a service with etcd.
	Register(s Service) error

	// Deregister a service with etcd.
	Deregister(s Service) error
}

type client struct {
	keysAPI etcd.KeysAPI
	ctx     context.Context
}

// ClientOptions defines options for the etcd client. All values are optional.
// If any duration is not specified, a default of 3 seconds will be used.
type ClientOptions struct {
	Cert                    string
	Key                     string
	CACert                  string
	DialTimeout             time.Duration
	DialKeepAlive           time.Duration
	HeaderTimeoutPerRequest time.Duration
}

// NewClient returns Client with a connection to the named machines. It will
// return an error if a connection to the cluster cannot be made. The parameter
// machines needs to be a full URL with schemas. e.g. "http://localhost:2379"
// will work, but "localhost:2379" will not.
func NewClient(ctx context.Context, machines []string, options ClientOptions) (Client, error) {
	if options.DialTimeout == 0 {
		options.DialTimeout = 3 * time.Second
	}
	if options.DialKeepAlive == 0 {
		options.DialKeepAlive = 3 * time.Second
	}
	if options.HeaderTimeoutPerRequest == 0 {
		options.HeaderTimeoutPerRequest = 3 * time.Second
	}

	transport := etcd.DefaultTransport
	if options.Cert != "" && options.Key != "" {
		tlsCert, err := tls.LoadX509KeyPair(options.Cert, options.Key)
		if err != nil {
			return nil, err
		}
		caCertCt, err := ioutil.ReadFile(options.CACert)
		if err != nil {
			return nil, err
		}
		caCertPool := x509.NewCertPool()
		caCertPool.AppendCertsFromPEM(caCertCt)
		transport = &http.Transport{
			TLSClientConfig: &tls.Config{
				Certificates: []tls.Certificate{tlsCert},
				RootCAs:      caCertPool,
			},
			Dial: func(network, address string) (net.Conn, error) {
				return (&net.Dialer{
					Timeout:   options.DialTimeout,
					KeepAlive: options.DialKeepAlive,
				}).Dial(network, address)
			},
		}
	}

	ce, err := etcd.New(etcd.Config{
		Endpoints:               machines,
		Transport:               transport,
		HeaderTimeoutPerRequest: options.HeaderTimeoutPerRequest,
	})
	if err != nil {
		return nil, err
	}

	return &client{
		keysAPI: etcd.NewKeysAPI(ce),
		ctx:     ctx,
	}, nil
}

// GetEntries implements the etcd Client interface.
func (c *client) GetEntries(key string) ([]string, error) {
	resp, err := c.keysAPI.Get(c.ctx, key, &etcd.GetOptions{Recursive: true})
	if err != nil {
		return nil, err
	}

	// Special case. Note that it's possible that len(resp.Node.Nodes) == 0 and
	// resp.Node.Value is also empty, in which case the key is empty and we
	// should not return any entries.
	if len(resp.Node.Nodes) == 0 && resp.Node.Value != "" {
		return []string{resp.Node.Value}, nil
	}

	entries := make([]string, len(resp.Node.Nodes))
	for i, node := range resp.Node.Nodes {
		entries[i] = node.Value
	}
	return entries, nil
}

// WatchPrefix implements the etcd Client interface.
func (c *client) WatchPrefix(prefix string, ch chan struct{}) {
	watch := c.keysAPI.Watcher(prefix, &etcd.WatcherOptions{AfterIndex: 0, Recursive: true})
	ch <- struct{}{} // make sure caller invokes GetEntries
	for {
		if _, err := watch.Next(c.ctx); err != nil {
			return
		}
		ch <- struct{}{}
	}
}

func (c *client) Register(s Service) error {
	if s.Key == "" {
		return ErrNoKey
	}
	if s.Value == "" {
		return ErrNoValue
	}
	var err error
	if s.TTL != nil {
		_, err = c.keysAPI.Set(c.ctx, s.Key, s.Value, &etcd.SetOptions{
			PrevExist: etcd.PrevIgnore,
			TTL:       s.TTL.ttl,
		})
	} else {
		_, err = c.keysAPI.Create(c.ctx, s.Key, s.Value)
	}
	return err
}

func (c *client) Deregister(s Service) error {
	if s.Key == "" {
		return ErrNoKey
	}
	_, err := c.keysAPI.Delete(c.ctx, s.Key, s.DeleteOptions)
	return err
}
