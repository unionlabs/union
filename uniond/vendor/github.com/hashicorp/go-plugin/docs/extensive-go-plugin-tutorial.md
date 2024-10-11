# Intro

If you don't know what go-plugin is, don't worry, here is a small introduction on the subject matter:

Back in the old days when Go didn't have the `plugin` package, HashiCorp was looking for a way to use plugins.

Mitchell had this brilliant idea of using RPC over the local network to serve a local interface as something that could easily be implemented with any other language that supported RPC. This sounds convoluted but has many benefits! For example, your code will never crash because of a plugin and the ability to use any language to implement a plugin. Not just Go.

It has been a battle-hardened solution for years now and is being actively used by Terraform, Vault, Consul, and especially Packer. All using go-plugin in order to provide a much needed flexibility. Writing a plugin is easy. Or so they say.

It can get complicated quickly, for example, if you are trying to use GRPC. You can lose sight of what exactly you'll need to implement, where and why; or utilizing various languages; or using go-plugins in your own project and extending your CLI with pluggable components.

These are all nothing to sneeze at. Suddenly you'll find yourself with hundreds of lines of code pasted from various examples and yet nothing works. Or worse, it DOES work but you have no idea how. Then you find yourself needing to extend it with a new capability, or you find an elusive bug and can't trace its origins.

Let's try to demystify things and draw a clearer picture about how it works and how the pieces fit together.

# Basic plugin

Start by writing a simple Go GRPC plugin. In fact, we can go through the basic example in the go-plugin’s repository. We'll go step-by-step, and the switch to GRPC will be much easier!

## Basic concepts

### Server

In the case of plugins, the Server is the one serving the plugin's implementation. This means the server will have to provide the implementation to an interface.

### Client

The Client calls the server in order to execute the desired behaviour. The underlying logic will connect to the server running on localhost on a random higher port, call the wanted function’s implementation and wait for a response. Once the response is received provide that back to the calling Client.

## Implementation

### The main function

#### Logger

The plugins defined here use stdout in a special way. If you aren't writing a Go based plugin, you will have to do that yourself by outputting something like this:

~~~
1|1|tcp|127.0.0.1:1234|grpc
~~~

We'll come back to this later. Suffice to say the framework will pick this up and will connect to the plugin based on the output. In order to get some output back, we must define a special logger:

~~~go
	// Create an hclog.Logger
	logger := hclog.New(&hclog.LoggerOptions{
		Name:   "plugin",
		Output: os.Stdout,
		Level:  hclog.Debug,
	})
~~~

#### NewClient

~~~go
	// We're a host! Start by launching the plugin process.
	client := plugin.NewClient(&plugin.ClientConfig{
		HandshakeConfig: handshakeConfig,
		Plugins:         pluginMap,
		Cmd:             exec.Command("./plugin/greeter"),
		Logger:          logger,
	})
	defer client.Kill()
~~~

What is happening here? Let's see one by one:

`HandshakeConfig: handshakeConfig,`: This part is the handshake configuration of the plugin. It has a nice comment as well.

~~~go
// handshakeConfigs are used to just do a basic handshake between
// a plugin and host. If the handshake fails, a user friendly error is shown.
// This prevents users from executing bad plugins or executing a plugin
// directory. It is a UX feature, not a security feature.
var handshakeConfig = plugin.HandshakeConfig{
	ProtocolVersion:  1,
	MagicCookieKey:   "BASIC_PLUGIN",
	MagicCookieValue: "hello",
}
~~~

The `ProtocolVersion` here is used in order to maintain compatibility with your current plugin versions. It's basically like an API version. If you increase this, you will have two options. Don't accept lower protocol versions or switch to the version number and use a different client implementation for a lower version than for a higher version. This way you will maintain backwards compatibility.

The `MagicCookieKey` and `MagicCookieValue` are used for a basic handshake which the comment is talking about. You have to set this **ONCE** for your application. Never change it again, for if you do, your plugins will no longer work. For uniqueness sake, I suggest using UUID.

`Cmd` is one of the most important parts about a plugin. Basically how plugins work is that they boil down to a compiled binary which starts an RPC server. This is where you will have to define the binary which will be executed and does all this. Since this is all happening locally, (please keep in mind that Go-plugins only support localhost, and for a good reason), these binaries will most likely sit next to your application's binary or in a pre-configured global location. Something like:  `~/.config/my-app/plugins`. This is individual for each plugin of course. The plugins can be autoloaded via a discovery function given a path and a glob.

And last but not least is the `Plugins` map. This map is used in order to identify a plugin called `Dispense`. This map is globally available and must stay consistent in order for all the plugins to work:

~~~go
// pluginMap is the map of plugins we can dispense.
var pluginMap = map[string]plugin.Plugin{"greeter": &shared.GreeterPlugin{}}
~~~

You can see that the key is the name of the plugin and the value is the plugin.

We then proceed to create an RPC client:

~~~go
	// Connect via RPC
	rpcClient, err := client.Client()
	if err != nil {
		log.Fatal(err)
	}
~~~

Nothing fancy about this one...

The interesting part is this:

~~~go
	// Request the plugin
	raw, err := rpcClient.Dispense("greeter")
	if err != nil {
		log.Fatal(err)
	}
~~~

Dispense will look in the above created map and search for the plugin. If it cannot find it, it will throw an error. If it does find it, it will cast this plugin to an RPC or a GRPC type plugin. Then proceed to create an RPC or a GRPC client out of it.

There is no call yet. This is just creating a client and parsing it to a respective representation.

Now, the magic:

~~~go
	// We should have a Greeter now! This feels like a normal interface
	// implementation but is in fact over an RPC connection.
	greeter := raw.(shared.Greeter)
	fmt.Println(greeter.Greet())
~~~

Here, we are type asserting our raw GRPC client into our own plugin type. This is so we can call the respective function on the plugin! Once that's done we will have a {client,struct,implementation} that can be called like a simple function.

The implementation right now comes from greeter_impl.go, but that will change once protoc makes an appearance.

Behind the scenes, go-plugin will do a bunch of hat tricks with multiplexing TCP connections as well as a remote procedure call to our plugin. Our plugin then will run the function, generate some kind of output, and will then send that back for the waiting client.

The client will then proceed to parse the message into a given response type and will then return it back to the client’s callee.

This concludes main.go for now.

### The Interface

Now let’s investigate the Interface. The interface is used to provide calling details. This interface will be what defines our plugins’ capabilities. How does our `Greeter` look like?

~~~go
// Greeter is the interface that we're exposing as a plugin.
type Greeter interface {
	Greet() string
}
~~~

This defines a function which will return a string typed value.

Now, we will need a couple of things for this to work. Firstly we need something which defines the RPC workings. go-plugin is working with `net/http` inside. It also uses something called Yamux for connection multiplexing, but we needn’t worry about this detail.

Implementing the RPC details looks like this:

~~~go
// Here is an implementation that talks over RPC
type GreeterRPC struct {
    client *rpc.Client
}

func (g *GreeterRPC) Greet() string {
	var resp string
	err := g.client.Call("Plugin.Greet", new(interface{}), &resp)
	if err != nil {
		// You usually want your interfaces to return errors. If they don't,
		// there isn't much other choice here.
		panic(err)
	}

	return resp
}
~~~

Here the GreeterRPC struct is an RPC specific implementation that will handle communication over RPC. This is the Client in this setup.

In case of gRPC, it would look like this:

~~~go
// GRPCClient is an implementation of KV that talks over RPC.
type GreeterGRPC struct{ client proto.GreeterClient }

func (m *GreeterGRPC) Greet() (string, error) {
    s, err := m.client.Greet(context.Background(), &proto.Empty{})
	return s, err
}
~~~

What's Proto and what is GreeterClient? GRPC uses Google's protoc library to serialize and unserialize data. `proto.GreeterClient` is generated Go code by protoc. This code is a skeleton for which implementation detail will be replaced on run time. Well, the actual result will be used and not replaced as such.

Back to our previous example. The RPC client calls a specific Plugin function called Greet for which the implementation will be provided by a Server that will be streamed back over the RPC protocol.

The server is pretty easy to follow:

~~~go
// Here is the RPC server that GreeterRPC talks to, conforming to
// the requirements of net/rpc
type GreeterRPCServer struct {
	// This is the real implementation
	Impl Greeter
}
~~~

Impl is the concrete implementation that will be called in the Server's implementation of the Greet plugin. Now we must define Greet on the RPCServer in order for it to be able to call the remote code. This looks like as follows:

~~~go
func (s *GreeterRPCServer) Greet(args interface{}, resp *string) error {
	*resp = s.Impl.Greet()
	return nil
}
~~~

This is all still boilerplate for the RPC works. Now comes the plugin. For this, the comment is actually quite good:

~~~go
// This is the implementation of plugin.Plugin so we can serve/consume this
//
// This has two methods: Server must return an RPC server for this plugin
// type. We construct a GreeterRPCServer for this.
//
// Client must return an implementation of our interface that communicates
// over an RPC client. We return GreeterRPC for this.
//
// Ignore MuxBroker. That is used to create more multiplexed streams on our
// plugin connection and is a more advanced use case.
type GreeterPlugin struct {
	// Impl Injection
	Impl Greeter
}

func (p *GreeterPlugin) Server(*plugin.MuxBroker) (interface{}, error) {
	return &GreeterRPCServer{Impl: p.Impl}, nil
}

func (GreeterPlugin) Client(b *plugin.MuxBroker, c *rpc.Client) (interface{}, error) {
	return &GreeterRPC{client: c}, nil
}
~~~

So, remember: `GreeterRPCServer` is the one calling the actual implementation while Client is receiving the result of that call. The `GreeterPlugin` has the `Greeter` interface embedded just like the `RPCServer`. We will use the `GreeterPlugin` as a struct in the plugin map. This is the plugin that we will actually use.

This is all still common stuff. These things will need to be visible for both. The plugin's implementation will use the interface to see what it needs to implement. The Client will use it to see what to call and what APIs are available. Like, `Greet`.

### The Implementation

In a completely separate package, but which still has access to the interface definition, this plugin could be like this:

~~~go
// Here is a real implementation of Greeter
type GreeterHello struct {
	logger hclog.Logger
}

func (g *GreeterHello) Greet() string {
	g.logger.Debug("message from GreeterHello.Greet")
	return "Hello!"
}
~~~

We create a struct and then add the function to it which is defined by the plugin's interface. This interface, since it's required by both parties, could well sit in a common package outside of both programs. Something like a SDK. Both code could import it and use it as a common dependency. This way we have separated the interface from the plugin **and** the calling client.

The `main` function could look something like this:

~~~go
logger := hclog.New(&hclog.LoggerOptions{
    Level:      hclog.Trace,
    Output:     os.Stderr,
    JSONFormat: true,
})

greeter := &GreeterHello{
    logger: logger,
}
// pluginMap is the map of plugins we can dispense.
var pluginMap = map[string]plugin.Plugin{
    "greeter": &shared.GreeterPlugin{Impl: greeter},
}

logger.Debug("message from plugin", "foo", "bar")

plugin.Serve(&plugin.ServeConfig{
    HandshakeConfig: handshakeConfig,
    Plugins:         pluginMap,
})
~~~

Notice two things that we need. One is the `handshakeConfig`. You can either define it here, with the same cookie details as you defined in the client code, or you can extract the handshake information into the SDK. This is up to you.

Then the next interesting thing is the `plugin.Serve` method. The plugins open up a RPC communication socket and over a hijacked `stdout`, broadcasts its availability to the calling Client in this format:

~~~bash
CORE-PROTOCOL-VERSION | APP-PROTOCOL-VERSION | NETWORK-TYPE | NETWORK-ADDR | PROTOCOL
~~~

For Go plugins, you don't have to concern yourself with this. `go-plugin` takes care of all this for you. For non-Go versions, we must take this into account. And before calling serve, we need to output this information to `stdout`.

For example, a Python plugin must deal with this himself. Like this:

~~~python
# Output information
print("1|1|tcp|127.0.0.1:1234|grpc")
sys.stdout.flush()
~~~

For GRPC plugins, it's also mandatory to implement a HealthChecker.

How would all this look like with GRPC?

It gets slightly more complicated but not too much. We need to use `protoc` to create a protocol description for our implementation, and then we will call that. Let's look at this now by converting the basic greeter example into GRPC.

# GRPC Basic plugin

The example that's under GRPC is quite elaborate and perhaps you don't need the Python part. I will focus on the basic RPC example into a GRPC example. That should not be a problem.

## The API

First and foremost, you will need to define an API to implement with `protoc`. For our basic example, the protoc file could look like this:

~~~proto3
syntax = "proto3";
package proto;

message GreetResponse {
    string message = 1;
}

message Empty {}

service GreeterService {
    rpc Greet(Empty) returns (GreetResponse);
}
~~~

The syntax is quite simple and readable. What this defines is a message, which is a response, that will contain a `message` with the type `string`. The `service` defines a service which has a method called `Greet`. The service definition is basically an interface for which we will be providing the concrete implementation through the plugin.

To read more about protoc, visit this page: [Google Protocol Buffer](https://developers.google.com/protocol-buffers/).

## Generate the code

Now, with the protoc definition in hand, we need to generate the stubs that the local client implementation can call. That client call will then, through the remote procedure call, call the right function on the server which will have the concrete implementation at the ready. Run it and return the result in the specified format. Because the stub needs to be available by both parties, (the client AND the server), this needs to live in a shared location.

The client is calling the stub and the server is implementing the stub. Both need it in order to know what to call/implement.

To generate the code, run the following command:

~~~bash
protoc -I proto/ proto/greeter.proto --go_out=plugins=grpc:proto
~~~

I encourage you to read the generated code. Much will make little sense at first. It will have a bunch of structs and defined things that the GRPC package will use in order to server the function. The interesting bits and pieces are:

~~~go
func (m *GreetResponse) GetMessage() string {
	if m != nil {
		return m.Message
	}
	return ""
}
~~~

Which will get use the message inside the response.

~~~go
type GreeterServiceClient interface {
	Greet(ctx context.Context, in *Empty, opts ...grpc.CallOption) (*GreetResponse, error)
}
~~~


This is our ServiceClient interface which defines the Greet function’s topology.

And lastly, this guy:

~~~go
func RegisterGreeterServiceServer(s *grpc.Server, srv GreeterServiceServer) {
	s.RegisterService(&_GreeterService_serviceDesc, srv)
}
~~~

Which we will need in order to register our implementation for the server. We can ignore the rest.

## The interface

Much like the RPC, we need to define an interface for the client and server to use. This must be in a shared place as both the server and the client need to know about it. You could put this into an SDK and your peers could just get the SDK and implement some function for define and done. The interface definition in the GRPC land could look something like this:

~~~go
// Greeter is the interface that we're exposing as a plugin.
type Greeter interface {
	Greet() string
}

// This is the implementation of plugin.GRPCPlugin so we can serve/consume this.
type GreeterGRPCPlugin struct {
	// GRPCPlugin must still implement the Plugin interface
	plugin.Plugin
	// Concrete implementation, written in Go. This is only used for plugins
	// that are written in Go.
	Impl Greeter
}

func (p *GreeterGRPCPlugin) GRPCServer(broker *plugin.GRPCBroker, s *grpc.Server) error {
	proto.RegisterGreeterServer(s, &GRPCServer{Impl: p.Impl})
	return nil
}

func (p *GreeterGRPCPlugin) GRPCClient(ctx context.Context, broker *plugin.GRPCBroker, c *grpc.ClientConn) (interface{}, error) {
	return &GRPCClient{client: proto.NewGreeterClient(c)}, nil
}
~~~

With this we have the Plugin's implementation for hashicorp what needed to be done. The plugin will call the underlying implementation and serve/consume the plugin. We can now write the GRPC part of it.

Please note that `proto` is a shared library too where the protocol stubs reside. That needs to be somewhere on the path or in a separate SDK of some sort, but it must be visible.

## Writing the GRPC Client

Firstly we define the grpc client struct:

~~~go
// GRPCClient is an implementation of Greeter that talks over RPC.
type GRPCClient struct{ client proto.GreeterClient }
~~~

Then we define how the client will call the remote function:

~~~go
func (m *GRPCClient) Greet() string {
	ret, _ := m.client.Greet(context.Background(), &proto.Empty{})
	return ret.Message
}
~~~

This will take the `client` in the `GRPCClient` and will call the method on it. Once that's done we will return to the result `Message` property which will be `Hello!`. `proto.Empty` is an empty struct; we use this if there is no parameter for a defined method or no return value. We can't just leave it blank. `protoc` needs to be told explicitly that there is no parameter or return value.

## Writing the GRPC Server

The server implementation will also be similar. We call `Impl` here which will have our concrete plugin implementation.

~~~go
// Here is the gRPC server that GRPCClient talks to.
type GRPCServer struct {
	// This is the real implementation
	Impl Greeter
}

func (m *GRPCServer) Greet(
	ctx context.Context,
	req *proto.Empty) *proto.GreeterResponse {
	v := m.Impl.Greet()
	return &proto.GreeterResponse{Message: v}
}
~~~

And we will use the `protoc` defined message response. `v` will have the response from `Greet` which will be `Hello!` provided by the concrete plugin's implementation. We then transform that into a protoc type by setting the `Message` property on the `GreeterResponse` struct provided by the automatically generated protoc stub code.

Easy, right?

## Writing the plugin itself

The whole thing looks much like the RPC implementation with just a few small modifications and changes. This can sit completely outside of everything, or can even be provided by a third party implementor.

~~~go
// Here is a real implementation of KV that writes to a local file with
// the key name and the contents are the value of the key.
type Greeter struct{}

func (Greeter) Greet() error {
	return "Hello!"
}

func main() {
	plugin.Serve(&plugin.ServeConfig{
		HandshakeConfig: shared.Handshake,
		Plugins: map[string]plugin.Plugin{
			"greeter": &shared.GreeterGRPCPlugin{Impl: &Greeter{}},
		},

		// A non-nil value here enables gRPC serving for this plugin...
		GRPCServer: plugin.DefaultGRPCServer,
	})
}
~~~

## Calling it all in the main

Once all that is done, the `main` function looks the same as RPC's main but with some small modifications.

~~~go
	// We're a host. Start by launching the plugin process.
	client := plugin.NewClient(&plugin.ClientConfig{
		HandshakeConfig: shared.Handshake,
		Plugins:         shared.PluginMap,
		Cmd:             exec.Command("./plugin/greeter"),
		AllowedProtocols: []plugin.Protocol{plugin.ProtocolGRPC},
	})
~~~

The `NewClient` now defines `AllowedProtocols` to be `ProtocolGRPC`. The rest is the same as before calling `Dispense` and value hinting the plugin to the correct type then calling `Greet()`.

# Conclusion

This is it. We made it! Now our plugin works over GRPC with a defined API by protoc. The plugin's implementation can live where ever we want it, but it needs some shared data. These are:

* The generated code by `protoc`
* The defined plugin interface
* The GRPC Server and Client

These need to be visible by both the Client and the Server. The Server here is the plugin. If you are planning on making people be able to extend your application with go-plugin, you should make these available as a separate SDK. So people won't have to include your whole project just to implement an interface and use protoc. In fact, you could also extract the `protoc` definition into a separate repository so that your SDK can also pull it in.

You will have three repositories:
* Your application
* The SDK providing the interface and the GRPC Server and Client implementation
* The protoc definition file and generated skeleton ( for Go based plugins ).

Other languages will have to generate their own protoc code, and includ it into the plugin; like the Python implementation example located here: [Go-plugin Python Example](https://github.com/hashicorp/go-plugin/tree/master/examples/grpc/plugin-python). Also, read this documentation carefully: [non-go go-plugin](https://github.com/hashicorp/go-plugin/blob/master/docs/guide-plugin-write-non-go.md). This document will also clarify what `1|1|tcp|127.0.0.1:1234|grpc` means and will dissipate the confusion around how plugins work.

Lastly, if you would like to have an in-depth explanation about how go-plugin came to be, watch this video by Mitchell:

[go-plugin explanation video](https://www.youtube.com/watch?v=SRvm3zQQc1Q).

That's it. I hope this has helped to clear the confusion around how to use go-plugin.
