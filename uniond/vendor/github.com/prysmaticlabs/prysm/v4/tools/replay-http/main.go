/*
*
Tool for replaying http requests from a file of base64 encoded, line-delimited
Go http raw requests. Credits to https://gist.github.com/kasey/c9e663eae5baebbf8fbe548c2b1d961b.
*/
package main

import (
	"bufio"
	"bytes"
	"encoding/base64"
	"flag"
	"io"
	"net/http"
	"net/url"
	"os"
	"path"

	log "github.com/sirupsen/logrus"
)

var (
	filePath = flag.String("file", "", "file of line-delimited, base64-encoded Go http requests")
	endpoint = flag.String("endpoint", "http://localhost:14268/api/traces", "host:port endpoint to make HTTP requests to")
)

func main() {
	flag.Parse()
	if *filePath == "" {
		log.Fatal("Must provide --file")
	}

	f, err := os.Open(path.Clean(*filePath))
	if err != nil {
		log.Fatal(err)
	}
	defer func() {
		if err := f.Close(); err != nil {
			log.WithError(err).Error("Could not close stdout file")
		}
	}()
	lr := bufio.NewReader(f)
	for {
		line, err := lr.ReadBytes([]byte("\n")[0])
		if err == io.EOF {
			os.Exit(0)
		}
		if err != nil {
			log.Fatal(err)
		}
		line = line[0 : len(line)-1]
		decoded := make([]byte, base64.StdEncoding.DecodedLen(len(line)))
		_, err = base64.StdEncoding.Decode(decoded, line)
		if err != nil {
			log.Fatal(err)
		}
		dbuf := bytes.NewBuffer(decoded)
		req, err := http.ReadRequest(bufio.NewReader(dbuf))
		if err != nil {
			log.Fatal(err)
		}
		parsed, err := url.Parse(*endpoint)
		if err != nil {
			log.Fatal(err)
		}
		req.URL = parsed
		req.RequestURI = ""
		log.Println(req)
		resp, err := http.DefaultClient.Do(req)
		if err != nil {
			log.Fatal(err)
		}
		respBuf := bytes.NewBuffer(nil)
		if err := resp.Write(respBuf); err != nil {
			log.Fatal(err)
		}
		log.Println(respBuf.String())
	}
}
