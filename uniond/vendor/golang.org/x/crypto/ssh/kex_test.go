// Copyright 2013 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package ssh

// Key exchange tests.

import (
	"crypto/rand"
	"fmt"
	"reflect"
	"sync"
	"testing"
)

// Runs multiple key exchanges concurrent to detect potential data races with
// kex obtained from the global kexAlgoMap.
// This test needs to be executed using the race detector in order to detect
// race conditions.
func TestKexes(t *testing.T) {
	type kexResultErr struct {
		result *kexResult
		err    error
	}

	for name, kex := range kexAlgoMap {
		t.Run(name, func(t *testing.T) {
			wg := sync.WaitGroup{}
			for i := 0; i < 3; i++ {
				wg.Add(1)
				go func() {
					defer wg.Done()
					a, b := memPipe()

					s := make(chan kexResultErr, 1)
					c := make(chan kexResultErr, 1)
					var magics handshakeMagics
					go func() {
						r, e := kex.Client(a, rand.Reader, &magics)
						a.Close()
						c <- kexResultErr{r, e}
					}()
					go func() {
						r, e := kex.Server(b, rand.Reader, &magics, testSigners["ecdsa"].(AlgorithmSigner), testSigners["ecdsa"].PublicKey().Type())
						b.Close()
						s <- kexResultErr{r, e}
					}()

					clientRes := <-c
					serverRes := <-s
					if clientRes.err != nil {
						t.Errorf("client: %v", clientRes.err)
					}
					if serverRes.err != nil {
						t.Errorf("server: %v", serverRes.err)
					}
					if !reflect.DeepEqual(clientRes.result, serverRes.result) {
						t.Errorf("kex %q: mismatch %#v, %#v", name, clientRes.result, serverRes.result)
					}
				}()
			}
			wg.Wait()
		})
	}
}

func BenchmarkKexes(b *testing.B) {
	type kexResultErr struct {
		result *kexResult
		err    error
	}

	for name, kex := range kexAlgoMap {
		b.Run(name, func(b *testing.B) {
			for i := 0; i < b.N; i++ {
				t1, t2 := memPipe()

				s := make(chan kexResultErr, 1)
				c := make(chan kexResultErr, 1)
				var magics handshakeMagics

				go func() {
					r, e := kex.Client(t1, rand.Reader, &magics)
					t1.Close()
					c <- kexResultErr{r, e}
				}()
				go func() {
					r, e := kex.Server(t2, rand.Reader, &magics, testSigners["ecdsa"].(AlgorithmSigner), testSigners["ecdsa"].PublicKey().Type())
					t2.Close()
					s <- kexResultErr{r, e}
				}()

				clientRes := <-c
				serverRes := <-s

				if clientRes.err != nil {
					panic(fmt.Sprintf("client: %v", clientRes.err))
				}
				if serverRes.err != nil {
					panic(fmt.Sprintf("server: %v", serverRes.err))
				}
			}
		})
	}
}
