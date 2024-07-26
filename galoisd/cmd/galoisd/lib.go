//go:build library
// +build library

package main

/*
   #include <stdlib.h>
   #include <stdio.h>
   #include <errno.h>
*/
import "C"

import (
	"bytes"
	"io"
	"log"
	"unsafe"

	mpc "github.com/consensys/gnark/backend/groth16/bn254/mpcsetup"
)

//export Phase2Verify
func Phase2Verify(phase2PreviousRaw *C.char, phase2ContribRaw *C.char, l C.int) bool {
	phase2PreviousPayload := C.GoBytes(unsafe.Pointer(phase2PreviousRaw), l)
	var previous mpc.Phase2
	err := readFromBuffer(phase2PreviousPayload, &previous)
	if err != nil {
		log.Printf("Failed to read phase2PreviousPayload: %v\n", err)
		return false
	}
	phase2ContribPayload := C.GoBytes(unsafe.Pointer(phase2ContribRaw), l)
	var contrib mpc.Phase2
	err = readFromBuffer(phase2ContribPayload, &contrib)
	if err != nil {
		log.Printf("Failed to read phase2ContribPayload: %v\n", err)
		return false
	}
	err = mpc.VerifyPhase2(&previous, &contrib)
	if err != nil {
		log.Printf("Failed to verify phase2 contribution: %v\n", err)
		return false
	}
	return true
}

//export Phase2Contribute
func Phase2Contribute(phase2PayloadRaw *C.char, phase2ContribRaw *C.char, l C.int) bool {
	phase2Payload := C.GoBytes(unsafe.Pointer(phase2PayloadRaw), l)
	phase2Contrib, err := phase2Contribute(phase2Payload)
	if err != nil {
		log.Printf("Failed to contribute %v\n", err)
		return false
	}
	phase2ContribOutput := unsafe.Slice(phase2ContribRaw, l)
	for i := 0; i < int(l); i++ {
		phase2ContribOutput[i] = C.char(phase2Contrib[i])
	}
	return true
}

func phase2Contribute(phase2Payload []byte) ([]byte, error) {
	var srs2 mpc.Phase2
	err := readFromBuffer(phase2Payload, &srs2)
	if err != nil {
		log.Printf("Failed to read phase2Payload: %v\n", err)
		return nil, err
	}
	srs2.Contribute()
	var phase2Output bytes.Buffer
	_, err = srs2.WriteTo(&phase2Output)
	if err != nil {
		log.Printf("Failed to write phase2Payload: %v\n", err)
		return nil, err
	}
	return phase2Output.Bytes(), nil
}

func readFromBuffer(buffer []byte, obj io.ReaderFrom) error {
	_, err := obj.ReadFrom(bytes.NewReader(buffer))
	return err
}

func main() {}
