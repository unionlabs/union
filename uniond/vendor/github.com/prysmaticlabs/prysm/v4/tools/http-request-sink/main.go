// Package main implements a simple, http-request-sink which writes
// incoming http request bodies to an append-only text file at a specified directory.
package main

import (
	"bytes"
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"path/filepath"
	"strconv"
	"time"

	"github.com/prysmaticlabs/prysm/v4/config/params"
)

func main() {
	port := flag.Int("port", 8080, "port to listen on")
	writeDirPath := flag.String("write-dir", "", "directory to write an append-only file")
	flag.Parse()
	if *writeDirPath == "" {
		log.Fatal("Needs a -write-dir path")
	}

	// If the file doesn't exist, create it, or append to the file.
	f, err := os.OpenFile(
		filepath.Join(*writeDirPath, "requests.log"),
		os.O_APPEND|os.O_CREATE|os.O_RDWR,
		params.BeaconIoConfig().ReadWritePermissions,
	)
	if err != nil {
		log.Println(err)
	}
	defer func() {
		if err = f.Close(); err != nil {
			log.Fatal(err)
		}
	}()

	http.HandleFunc("/", func(writer http.ResponseWriter, r *http.Request) {
		reqContent := map[string]interface{}{}
		if err = parseRequest(r, &reqContent); err != nil {
			log.Println(err)
		}
		log.Printf("Capturing request from %s", r.RemoteAddr)
		if err = captureRequest(f, reqContent); err != nil {
			log.Println(err)
		}
	})
	log.Printf("Listening on port %d", *port)
	srv := &http.Server{
		Addr:              ":" + strconv.Itoa(*port),
		ReadHeaderTimeout: 3 * time.Second,
	}
	log.Fatal(srv.ListenAndServe())
}

func captureRequest(f *os.File, m map[string]interface{}) error {
	enc, err := json.Marshal(m)
	if err != nil {
		return err
	}
	_, err = f.WriteString(fmt.Sprintf("%s\n", enc))
	return err
}

func parseRequest(req *http.Request, unmarshalStruct interface{}) error {
	body, err := io.ReadAll(req.Body)
	if err != nil {
		return err
	}
	if err = req.Body.Close(); err != nil {
		return err
	}
	req.Body = io.NopCloser(bytes.NewBuffer(body))
	return json.Unmarshal(body, unmarshalStruct)
}
