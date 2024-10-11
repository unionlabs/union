package main

import (
	"bufio"
	"fmt"
	"io"
	"log"
	"reflect"

	"github.com/cometbft/cometbft/abci/types"
	cmtnet "github.com/cometbft/cometbft/internal/net"
)

func main() {
	conn, err := cmtnet.Connect("unix://test.sock")
	if err != nil {
		log.Fatal(err.Error())
	}

	// Make a bunch of requests
	counter := 0
	for i := 0; ; i++ {
		req := types.ToEchoRequest("foobar")
		_, err := makeRequest(conn, req)
		if err != nil {
			log.Fatal(err.Error())
		}
		counter++
		if counter%1000 == 0 {
			fmt.Println(counter)
		}
	}
}

func makeRequest(conn io.ReadWriter, req *types.Request) (*types.Response, error) {
	bufWriter := bufio.NewWriter(conn)

	// Write desired request
	err := types.WriteMessage(req, bufWriter)
	if err != nil {
		return nil, err
	}
	err = types.WriteMessage(types.ToFlushRequest(), bufWriter)
	if err != nil {
		return nil, err
	}
	err = bufWriter.Flush()
	if err != nil {
		return nil, err
	}

	// Read desired response
	res := &types.Response{}
	err = types.ReadMessage(conn, res)
	if err != nil {
		return nil, err
	}
	resFlush := &types.Response{}
	err = types.ReadMessage(conn, resFlush)
	if err != nil {
		return nil, err
	}
	if _, ok := resFlush.Value.(*types.Response_Flush); !ok {
		return nil, fmt.Errorf("expected flush response but got something else: %v", reflect.TypeOf(resFlush))
	}

	return res, nil
}
