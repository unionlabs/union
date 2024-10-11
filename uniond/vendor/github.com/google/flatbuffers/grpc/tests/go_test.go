package testing

import (
	flatbuffers "github.com/google/flatbuffers/go"
	"github.com/google/flatbuffers/tests/MyGame/Example"

	"context"
	"net"
	"testing"

	"google.golang.org/grpc"
	"google.golang.org/grpc/encoding"
)

type server struct {
	Example.UnimplementedMonsterStorageServer
}

// test used to send and receive in grpc methods
var test = "Flatbuffers"
var addr = "0.0.0.0:50051"

// gRPC server store method
func (s *server) Store(context context.Context, in *Example.Monster) (*flatbuffers.Builder, error) {
	b := flatbuffers.NewBuilder(0)
	i := b.CreateString(test)
	Example.StatStart(b)
	Example.StatAddId(b, i)
	b.Finish(Example.StatEnd(b))
	return b, nil

}

// gRPC server retrieve method
func (s *server) Retrieve(context context.Context, in *Example.Stat) (*flatbuffers.Builder, error) {
	b := flatbuffers.NewBuilder(0)
	i := b.CreateString(test)
	Example.MonsterStart(b)
	Example.MonsterAddName(b, i)
	b.Finish(Example.MonsterEnd(b))
	return b, nil
}

func StoreClient(c Example.MonsterStorageClient, t *testing.T) {
	b := flatbuffers.NewBuilder(0)
	i := b.CreateString(test)
	Example.MonsterStart(b)
	Example.MonsterAddName(b, i)
	b.Finish(Example.MonsterEnd(b))
	out, err := c.Store(context.Background(), b)
	if err != nil {
		t.Fatalf("Store client failed: %v", err)
	}
	if string(out.Id()) != test {
		t.Errorf("StoreClient failed: expected=%s, got=%s\n", test, out.Id())
		t.Fail()
	}
}

func RetrieveClient(c Example.MonsterStorageClient, t *testing.T) {
	b := flatbuffers.NewBuilder(0)
	i := b.CreateString(test)
	Example.StatStart(b)
	Example.StatAddId(b, i)
	b.Finish(Example.StatEnd(b))
	out, err := c.Retrieve(context.Background(), b)
	if err != nil {
		t.Fatalf("Retrieve client failed: %v", err)
	}
	monster, err := out.Recv()
	if err != nil {
		t.Fatalf("Recv failed: %v", err)
	}
	if string(monster.Name()) != test {
		t.Errorf("RetrieveClient failed: expected=%s, got=%s\n", test, monster.Name())
		t.Fail()
	}
}

func TestGRPC(t *testing.T) {
	lis, err := net.Listen("tcp", addr)
	if err != nil {
		t.Fatalf("Failed to listen: %v", err)
	}
	ser := grpc.NewServer()
	encoding.RegisterCodec(flatbuffers.FlatbuffersCodec{})
	Example.RegisterMonsterStorageServer(ser, &server{})
	go func() {
		if err := ser.Serve(lis); err != nil {
			t.Fatalf("Failed to serve: %v", err)
			t.FailNow()
		}
	}()
	conn, err := grpc.Dial(addr, grpc.WithInsecure(), grpc.WithCodec(flatbuffers.FlatbuffersCodec{}))
	if err != nil {
		t.Fatalf("Failed to connect: %v", err)
	}
	defer conn.Close()
	client := Example.NewMonsterStorageClient(conn)
	StoreClient(client, t)
	RetrieveClient(client, t)
}
