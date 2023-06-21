package bls
/*
#cgo CFLAGS:-I${SRCDIR}./include -DBLS_ETH
#include <mcl/bn_c384_256.h>
#include <bls/bls.h>
*/
import "C"
import (
	"crypto/rand"
	"fmt"
	"runtime"
	"unsafe"
)

// MultiVerify --
// true if all (sigs[i], pubs[i], concatenatedMsg[msgSize*i:msgSize*(i+1)]) are valid
// concatenatedMsg has the size of len(sigs) * 32
func MultiVerify(sigs []Sign, pubs []PublicKey, concatenatedMsg []byte) bool {
	msgSize := 32
	randSize := 8
	threadN := runtime.NumCPU()
	n := len(sigs)
	if n == 0 || len(pubs) != n || len(concatenatedMsg) != n*msgSize {
		return false
	}
	randVec := make([]byte, n*randSize)
	rand.Read(randVec)

	var e C.mclBnGT
	var aggSig Sign
	msg := uintptr(unsafe.Pointer(&concatenatedMsg[0]))
	rp := uintptr(unsafe.Pointer(&randVec[0]))

	maxThreadN := 32
	if threadN > maxThreadN {
		threadN = maxThreadN
	}
	minN := 16
	if threadN > 1 && n >= minN {
		et := make([]C.mclBnGT, threadN)
		aggSigt := make([]Sign, threadN)
		blockN := n / minN
		q := blockN / threadN
		r := blockN % threadN
		cs := make(chan int, threadN)
		sub := func(i int, sigs []Sign, pubs []PublicKey, msg uintptr, rp uintptr, m int) {
			C.blsMultiVerifySub(&et[i], &aggSigt[i].v, &sigs[0].v, &pubs[0].v, (*C.char)(unsafe.Pointer(msg)), C.mclSize(msgSize), (*C.char)(unsafe.Pointer(rp)), C.mclSize(randSize), C.mclSize(m))
			cs <- 1
		}
		for i := 0; i < threadN; i++ {
			m := q
			if r > 0 {
				m++
				r--
			}
			if m == 0 {
				threadN = i // n is too small for threadN
				break
			}
			m *= minN
			if i == threadN-1 {
				m = n // remain all
			}
			// C.blsMultiVerifySub(&et[i], &aggSigt[i].v, &sigs[0].v, &pubs[0].v, (*C.char)(unsafe.Pointer(msg)), C.mclSize(msgSize), (*C.char)(unsafe.Pointer(rp)), C.mclSize(randSize), C.mclSize(m))
			go sub(i, sigs, pubs, msg, rp, m)
			sigs = sigs[m:]
			pubs = pubs[m:]
			msg += uintptr(msgSize * m)
			rp += uintptr(randSize * m)
			n -= m
		}
		for i := 0; i < threadN; i++ {
			<-cs
		}
		e = et[0]
		aggSig = aggSigt[0]
		for i := 1; i < threadN; i++ {
			C.mclBnGT_mul(&e, &e, &et[i])
			aggSig.Add(&aggSigt[i])
		}
	} else {
		C.blsMultiVerifySub(&e, &aggSig.v, &sigs[0].v, &pubs[0].v, (*C.char)(unsafe.Pointer(msg)), C.mclSize(msgSize), (*C.char)(unsafe.Pointer(rp)), C.mclSize(randSize), C.mclSize(n))
	}
	return C.blsMultiVerifyFinal(&e, &aggSig.v) == 1
}

// SerializeUncompressed --
func (pub *PublicKey) SerializeUncompressed() []byte {
	buf := make([]byte, 96)
	// #nosec
	n := C.blsPublicKeySerializeUncompressed(unsafe.Pointer(&buf[0]), C.mclSize(len(buf)), &pub.v)
	if n == 0 {
		panic("err blsPublicKeySerializeUncompressed")
	}
	return buf[:n]
}

// SerializeUncompressed --
func (sig *Sign) SerializeUncompressed() []byte {
	buf := make([]byte, 192)
	// #nosec
	n := C.blsSignatureSerializeUncompressed(unsafe.Pointer(&buf[0]), C.mclSize(len(buf)), &sig.v)
	if n == 0 {
		panic("err blsSignatureSerializeUncompressed")
	}
	return buf[:n]
}

// DeserializeUncompressed --
func (pub *PublicKey) DeserializeUncompressed(buf []byte) error {
	// #nosec
	err := C.blsPublicKeyDeserializeUncompressed(&pub.v, unsafe.Pointer(&buf[0]), C.mclSize(len(buf)))
	if err == 0 {
		return fmt.Errorf("err blsPublicKeyDeserializeUncompressed %x", buf)
	}
	return nil
}

// DeserializeUncompressed --
func (sig *Sign) DeserializeUncompressed(buf []byte) error {
	// #nosec
	err := C.blsSignatureDeserializeUncompressed(&sig.v, unsafe.Pointer(&buf[0]), C.mclSize(len(buf)))
	if err == 0 {
		return fmt.Errorf("err blsSignatureDeserializeUncompressed %x", buf)
	}
	return nil
}

// SetETHmode --
// 0 ; old version
// 1 ; draft 05
// 2 ; draft 06
func SetETHmode(mode int) error {
	if err := C.blsSetETHmode(C.int(mode)); err != 0 {
		return fmt.Errorf("got non-zero response code: %d", err)
	}
	return nil
}

// AreAllMsgDifferent checks the given message slice to ensure that each 32 byte segment is unique.
func AreAllMsgDifferent(msgVec []byte) bool {
	const MSG_SIZE = 32
	n := len(msgVec) / MSG_SIZE
	if n*MSG_SIZE != len(msgVec) {
		return false
	}
	set := make(map[[MSG_SIZE]byte]struct{}, n)
	msg := [MSG_SIZE]byte{}
	for i := 0; i < n; i++ {
		// one copy can be reduced by unsafe.Pointer
		// msg := *(*[MSG_SIZE]byte)(unsafe.Pointer(&msgVec[i*MSG_SIZE : (i+1)*MSG_SIZE][0]))
		copy(msg[:], msgVec[i*MSG_SIZE:(i+1)*MSG_SIZE])
		_, ok := set[msg]
		if ok {
			return false
		}
		set[msg] = struct{}{}
	}
	return true
}

func (sig *Sign) innerAggregateVerify(pubVec []PublicKey, msgVec []byte, checkMessage bool) bool {
	const MSG_SIZE = 32
	n := len(pubVec)
	if n == 0 || len(msgVec) != MSG_SIZE*n {
		return false
	}
	if checkMessage && !AreAllMsgDifferent(msgVec) {
		return false
	}
	return C.blsAggregateVerifyNoCheck(&sig.v, &pubVec[0].v, unsafe.Pointer(&msgVec[0]), MSG_SIZE, C.mclSize(n)) == 1
}

// AggregateVerify --
// len(msgVec) == 32 * len(pubVec)
func (sig *Sign) AggregateVerifyNoCheck(pubVec []PublicKey, msgVec []byte) bool {
	return sig.innerAggregateVerify(pubVec, msgVec, false)
}

// AggregateVerify --
// len(msgVec) == 32 * len(pubVec)
// check all msgs are different each other
func (sig *Sign) AggregateVerify(pubVec []PublicKey, msgVec []byte) bool {
	return sig.innerAggregateVerify(pubVec, msgVec, true)
}
