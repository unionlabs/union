package etcdv3

import (
	"context"
	"io"
	"time"

	"github.com/go-kit/kit/endpoint"
	"github.com/go-kit/kit/sd"
	"github.com/go-kit/kit/sd/lb"
	"github.com/go-kit/log"
	"google.golang.org/grpc"
)

func Example() {
	// Let's say this is a service that means to register itself.
	// First, we will set up some context.
	var (
		etcdServer = "10.0.0.1:2379"      // in the change from v2 to v3, the schema is no longer necessary if connecting directly to an etcd v3 instance
		prefix     = "/services/foosvc/"  // known at compile time
		instance   = "1.2.3.4:8080"       // taken from runtime or platform, somehow
		key        = prefix + instance    // should be globally unique
		value      = "http://" + instance // based on our transport
		ctx        = context.Background()
	)

	options := ClientOptions{
		// Path to trusted ca file
		CACert: "",

		// Path to certificate
		Cert: "",

		// Path to private key
		Key: "",

		// Username if required
		Username: "",

		// Password if required
		Password: "",

		// If DialTimeout is 0, it defaults to 3s
		DialTimeout: time.Second * 3,

		// If DialKeepAlive is 0, it defaults to 3s
		DialKeepAlive: time.Second * 3,

		// If passing `grpc.WithBlock`, dial connection will block until success.
		DialOptions: []grpc.DialOption{grpc.WithBlock()},
	}

	// Build the client.
	client, err := NewClient(ctx, []string{etcdServer}, options)
	if err != nil {
		panic(err)
	}

	// Build the registrar.
	registrar := NewRegistrar(client, Service{
		Key:   key,
		Value: value,
	}, log.NewNopLogger())

	// Register our instance.
	registrar.Register()

	// At the end of our service lifecycle, for example at the end of func main,
	// we should make sure to deregister ourselves. This is important! Don't
	// accidentally skip this step by invoking a log.Fatal or os.Exit in the
	// interim, which bypasses the defer stack.
	defer registrar.Deregister()

	// It's likely that we'll also want to connect to other services and call
	// their methods. We can build an Instancer to listen for changes from etcd,
	// create Endpointer, wrap it with a load-balancer to pick a single
	// endpoint, and finally wrap it with a retry strategy to get something that
	// can be used as an endpoint directly.
	barPrefix := "/services/barsvc"
	logger := log.NewNopLogger()
	instancer, err := NewInstancer(client, barPrefix, logger)
	if err != nil {
		panic(err)
	}
	endpointer := sd.NewEndpointer(instancer, barFactory, logger)
	balancer := lb.NewRoundRobin(endpointer)
	retry := lb.Retry(3, 3*time.Second, balancer)

	// And now retry can be used like any other endpoint.
	req := struct{}{}
	if _, err = retry(ctx, req); err != nil {
		panic(err)
	}
}

func barFactory(string) (endpoint.Endpoint, io.Closer, error) { return endpoint.Nop, nil, nil }
