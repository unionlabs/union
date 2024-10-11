/*
 * Copyright 2020 Dgraph Labs, Inc. and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package main

// #include <stdlib.h>
import "C"
import (
	"fmt"
	"log"
	"math/rand"
	"net/http"
	_ "net/http/pprof"
	"os"
	"os/signal"
	"runtime"
	"sync/atomic"
	"syscall"
	"time"
	"unsafe"

	"github.com/dustin/go-humanize"

	"github.com/dgraph-io/ristretto/z"
)

type S struct {
	key  uint64
	val  []byte
	next *S
	inGo bool
}

var (
	ssz      = int(unsafe.Sizeof(S{}))
	lo, hi   = int64(1 << 30), int64(16 << 30)
	increase = true
	stop     int32
	fill     []byte
	maxMB    = 32

	cycles int64 = 16
)
var numbytes int64
var counter int64

func newS(sz int) *S {
	var s *S
	if b := Calloc(ssz); len(b) > 0 {
		s = (*S)(unsafe.Pointer(&b[0]))
	} else {
		s = &S{inGo: true}
	}

	s.val = Calloc(sz)
	copy(s.val, fill)
	if s.next != nil {
		log.Fatalf("news.next must be nil: %p", s.next)
	}
	return s
}

func freeS(s *S) {
	Free(s.val)
	if !s.inGo {
		buf := (*[z.MaxArrayLen]byte)(unsafe.Pointer(s))[:ssz:ssz]
		Free(buf)
	}
}

func (s *S) allocateNext(sz int) {
	ns := newS(sz)
	s.next, ns.next = ns, s.next
}

func (s *S) deallocNext() {
	if s.next == nil {
		log.Fatal("next should not be nil")
	}
	next := s.next
	s.next = next.next
	freeS(next)
}

func memory() {
	// In normal mode, z.NumAllocBytes would always be zero. So, this program would misbehave.
	curMem := NumAllocBytes()
	if increase {
		if curMem > hi {
			increase = false
		}
	} else {
		if curMem < lo {
			increase = true
			runtime.GC()
			time.Sleep(3 * time.Second)

			counter++
		}
	}
	var js z.MemStats
	z.ReadMemStats(&js)

	fmt.Printf("[%d] Current Memory: %s. Increase? %v, MemStats [Active: %s, Allocated: %s,"+
		" Resident: %s, Retained: %s]\n",
		counter, humanize.IBytes(uint64(curMem)), increase,
		humanize.IBytes(js.Active), humanize.IBytes(js.Allocated),
		humanize.IBytes(js.Resident), humanize.IBytes(js.Retained))
}

func viaLL() {
	ticker := time.NewTicker(10 * time.Millisecond)
	defer ticker.Stop()

	root := newS(1)
	for range ticker.C {
		if counter >= cycles {
			fmt.Printf("Finished %d cycles. Deallocating...\n", counter)
			break
		}
		if atomic.LoadInt32(&stop) == 1 {
			break
		}
		if increase {
			root.allocateNext(rand.Intn(maxMB) << 20)
		} else {
			root.deallocNext()
		}
		memory()
	}
	for root.next != nil {
		root.deallocNext()
		memory()
	}
	freeS(root)
}

func main() {
	check()
	fill = make([]byte, maxMB<<20)
	_, _ = rand.Read(fill)

	c := make(chan os.Signal, 10)
	signal.Notify(c, os.Interrupt, syscall.SIGTERM)
	go func() {
		<-c
		fmt.Println("Stopping")
		atomic.StoreInt32(&stop, 1)
	}()
	go func() {
		if err := http.ListenAndServe("0.0.0.0:8080", nil); err != nil {
			log.Fatalf("Error: %v", err)
		}
	}()

	viaLL()
	if left := NumAllocBytes(); left != 0 {
		log.Fatalf("Unable to deallocate all memory: %v\n", left)
	}
	runtime.GC()
	fmt.Println("Done. Reduced to zero memory usage.")
	time.Sleep(5 * time.Second)
}
