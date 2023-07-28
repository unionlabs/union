package strobe

/***************************************************/
/*
/* This is a compact implementation of Strobe.
/* As it hasn't been thoroughly tested only use this
/* for experimental purposes :)
/*
/* Author: David Wong
/* Contact: www.cryptologie.net/contact
/*
/***************************************************/

import (
	"bytes"
	"encoding/binary"
	"encoding/hex"
)

const (
	// The size of the authentication tag used in AEAD functions
	MACLEN = 16
)

// KEY inserts a key into the state.
// It also provides forward secrecy.
func (s *Strobe) KEY(key []byte) {
	s.Operate(false, "KEY", key, 0, false)
}

// PRF provides a hash of length `output_len` of all previous operations
// It can also be used to generate random numbers, it is forward secure.
func (s *Strobe) PRF(outputLen int) []byte {
	return s.Operate(false, "PRF", []byte{}, outputLen, false)
}

// Send_ENC_unauthenticated is used to encrypt some plaintext
// it should be followed by Send_MAC in order to protect its integrity
// `meta` is used for encrypted framing data.
func (s *Strobe) Send_ENC_unauthenticated(meta bool, plaintext []byte) []byte {
	return s.Operate(meta, "send_ENC", plaintext, 0, false)
}

// Recv_ENC_unauthenticated is used to decrypt some received ciphertext
// it should be followed by Recv_MAC in order to protect its integrity
// `meta` is used for decrypting framing data.
func (s *Strobe) Recv_ENC_unauthenticated(meta bool, ciphertext []byte) []byte {
	return s.Operate(meta, "recv_ENC", ciphertext, 0, false)
}

// AD allows you to authenticate Additional Data
// it should be followed by a Send_MAC or Recv_MAC in order to truly work
func (s *Strobe) AD(meta bool, additionalData []byte) {
	s.Operate(meta, "AD", additionalData, 0, false)
}

// Send_CLR allows you to send data in cleartext
// `meta` is used to send framing data
func (s *Strobe) Send_CLR(meta bool, cleartext []byte) {
	s.Operate(meta, "send_CLR", cleartext, 0, false)
}

// Recv_CLR allows you to receive data in cleartext.
// `meta` is used to receive framing data
func (s *Strobe) Recv_CLR(meta bool, cleartext []byte) {
	s.Operate(meta, "recv_CLR", cleartext, 0, false)
}

// Send_MAC allows you to produce an authentication tag.
// `meta` is appropriate for checking the integrity of framing data.
func (s *Strobe) Send_MAC(meta bool, output_length int) []byte {
	return s.Operate(meta, "send_MAC", []byte{}, output_length, false)
}

// Recv_MAC allows you to verify a received authentication tag.
// `meta` is appropriate for checking the integrity of framing data.
func (s *Strobe) Recv_MAC(meta bool, MAC []byte) bool {
	if s.Operate(meta, "recv_MAC", MAC, 0, false)[0] == 0 {
		return true
	}
	return false
}

// RATCHET allows you to introduce forward secrecy in a protocol.
func (s *Strobe) RATCHET(length int) {
	s.Operate(false, "RATCHET", []byte{}, length, false)
}

// Send_AEAD allows you to encrypt data and authenticate additional data
// It is similar to AES-GCM.
func (s *Strobe) Send_AEAD(plaintext, ad []byte) (ciphertext []byte) {
	ciphertext = append(ciphertext, s.Send_ENC_unauthenticated(false, plaintext)...)
	s.AD(false, ad)
	ciphertext = append(ciphertext, s.Send_MAC(false, MACLEN)...)
	return
}

// Recv_AEAD allows you to decrypt data and authenticate additional data
// It is similar to AES-GCM.
func (s *Strobe) Recv_AEAD(ciphertext, ad []byte) (plaintext []byte, ok bool) {
	if len(ciphertext) < MACLEN {
		ok = false
		return
	}
	plaintext = s.Recv_ENC_unauthenticated(false, ciphertext[:len(ciphertext)-MACLEN])
	s.AD(false, ad)
	ok = s.Recv_MAC(false, ciphertext[len(ciphertext)-MACLEN:])
	return
}

//
// Strobe Objects
//

type role uint8 // for strobe.I0

const (
	iInitiator role = iota // set if we send the first transport message
	iResponder             // set if we receive the first transport message
	iNone                  // starting value
)

/*
  We do not use strobe's `pos` variable here since it is easily
  obtainable via `len(buf)`
*/
// TODO: accept permutations of different sizes
type Strobe struct {
	// config
	duplexRate int // 1600/8 - security/4
	StrobeR    int // duplexRate - 2

	// strobe specific
	initialized bool  // used to avoid padding during the first permutation
	posBegin    uint8 // start of the current operation (0 := previous block)
	I0          role

	// streaming API
	curFlags flag

	// duplex construction (see sha3.go)
	a            [25]uint64 // the actual state
	buf          []byte     // a pointer into the storage, it also serves as `pos` variable
	storage      []byte     // to-be-XORed (used for optimizations purposes)
	tempStateBuf []byte     // utility slice used for temporary duplexing operations
}

// Clone allows you to clone a Strobe state.
func (s Strobe) Clone() *Strobe {
	ret := s
	// need to recreate some buffers
	ret.storage = make([]byte, s.duplexRate)
	copy(ret.storage, s.storage)
	ret.tempStateBuf = make([]byte, s.duplexRate)
	copy(ret.tempStateBuf, s.tempStateBuf)
	// and set pointers
	ret.buf = ret.storage[:len(ret.buf)]
	return &ret
}

// Serialize allows one to serialize the strobe state to later recover it.
// [security(1)|initialized(1)|I0(1)|curFlags(1)|posBegin(1)|pos(1)|[25]uint64 state]
func (s Strobe) Serialize() []byte {
	// serialized data
	serialized := make([]byte, 6+25*8) // TODO: this is only for keccak-f[1600]
	// security?
	security := (1600/8 - s.duplexRate) * 4
	if security == 128 {
		serialized[0] = 0
	} else {
		serialized[0] = 1
	}
	// initialized?
	if s.initialized {
		serialized[1] = 1
	} else {
		serialized[1] = 0
	}
	// I0
	serialized[2] = byte(s.I0)
	// curFlags
	serialized[3] = byte(s.curFlags)
	// posBegin
	serialized[4] = byte(s.posBegin)
	// pos
	serialized[5] = byte(len(s.buf))
	// make sure to XOR what's left to XOR in the storage
	var buf [1600 / 8]byte
	var state [25]uint64
	copy(buf[:len(s.buf)], s.storage[:len(s.buf)]) // len(s.buf) = pos
	copy(state[:], s.a[:])
	xorState(&state, buf[:])
	// state
	var b []byte
	b = serialized[6:]
	for i := 0; len(b) >= 8; i++ {
		binary.LittleEndian.PutUint64(b, state[i])
		b = b[8:]
	}
	//
	return serialized
}

// Recover state allows one to re-create a strobe state from a serialized state.
// [security(1)|initialized(1)|I0(1)|curFlags(1)|posBegin(1)|pos(1)|[25]uint64 state]
func RecoverState(serialized []byte) (s Strobe) {
	if len(serialized) != 6+25*8 {
		panic("strobe: cannot recover state of invalid length")
	}
	// security?
	if serialized[0] > 1 {
		panic("strobe: cannot recover state with invalid security")
	}
	security := 128
	if security == 1 {
		security = 256
	}
	// init vars from security
	s.duplexRate = 1600/8 - security/4
	s.StrobeR = s.duplexRate - 2
	// need to recreate some buffers
	s.storage = make([]byte, s.duplexRate)
	s.tempStateBuf = make([]byte, s.duplexRate)
	// initialized?
	if serialized[1] == 1 {
		s.initialized = true
	} else {
		s.initialized = false
	}
	// I0?
	if serialized[2] > 3 {
		panic("strobe: cannot recover state with invalid role")
	}
	s.I0 = role(serialized[2])
	// curFlags + posBegin
	s.curFlags = flag(serialized[3])
	s.posBegin = uint8(serialized[4])
	// pos
	pos := int(serialized[5])
	s.buf = s.storage[:pos]
	// state
	serialized = serialized[6:]
	for i := 0; i < 25; i++ {
		a := binary.LittleEndian.Uint64(serialized[:8])
		s.a[i] = a
		serialized = serialized[8:]
	}
	//
	return
}

//
// Flags
//

type flag uint8

const (
	flagI flag = 1 << iota
	flagA
	flagC
	flagT
	flagM
	flagK
)

var operationMap = map[string]flag{
	"AD":       flagA,
	"KEY":      flagA | flagC,
	"PRF":      flagI | flagA | flagC,
	"send_CLR": flagA | flagT,
	"recv_CLR": flagI | flagA | flagT,
	"send_ENC": flagA | flagC | flagT,
	"recv_ENC": flagI | flagA | flagC | flagT,
	"send_MAC": flagC | flagT,
	"recv_MAC": flagI | flagC | flagT,
	"RATCHET":  flagC,
}

//
// Helper
//

// this only works for 8-byte alligned buffers
func xorState(state *[25]uint64, buf []byte) {
	n := len(buf) / 8
	for i := 0; i < n; i++ {
		a := binary.LittleEndian.Uint64(buf)
		state[i] ^= a
		buf = buf[8:]
	}
}

// this only works for 8-byte alligned buffers
func outState(state [25]uint64, b []byte) {
	for i := 0; len(b) >= 8; i++ {
		binary.LittleEndian.PutUint64(b, state[i])
		b = b[8:]
	}
}

// since the golang implementation does not absorb
// things in the state "right away" (sometimes just
// wait for the buffer to fill) we need a function
// to properly print the state even when the state
// is in this "temporary" state.
func (s Strobe) debugPrintState() string {
	// copy _storage into buf
	var buf [1600 / 8]byte
	copy(buf[:len(s.buf)], s.storage[:len(s.buf)])
	// copy _state into state
	var state [25]uint64
	copy(state[:], s.a[:])
	// xor
	xorState(&state, buf[:])
	// print
	outState(state, buf[:])
	return hex.EncodeToString(buf[:])
}

//
// Core functions
//

// InitStrobe allows you to initialize a new strobe instance with a customization string (that can be empty) and a security target (either 128 or 256).
func InitStrobe(customizationString string, security int) (s Strobe) {
	// compute security and rate
	if security != 128 && security != 256 {
		panic("strobe: security must be set to either 128 or 256")
	}
	s.duplexRate = 1600/8 - security/4
	s.StrobeR = s.duplexRate - 2
	// init vars
	s.storage = make([]byte, s.duplexRate)
	s.tempStateBuf = make([]byte, s.duplexRate)
	s.I0 = iNone
	s.initialized = false
	// absorb domain + initialize + absorb custom string
	domain := []byte{1, byte(s.StrobeR + 2), 1, 0, 1, 12 * 8}
	domain = append(domain, []byte("STROBEv1.0.2")...)
	s.buf = s.storage[:0]
	s.duplex(domain, false, false, true)
	s.initialized = true
	s.Operate(true, "AD", []byte(customizationString), 0, false)

	return
}

// runF: applies the STROBE's + cSHAKE's padding and the Keccak permutation
func (s *Strobe) runF() {
	if s.initialized {
		// if we're initialize we apply the strobe padding
		if len(s.buf) > s.StrobeR {
			panic("strobe: buffer is never supposed to reach strobeR")
		}
		s.buf = append(s.buf, s.posBegin)
		s.buf = append(s.buf, 0x04)
		zerosStart := len(s.buf)
		s.buf = s.storage[:s.duplexRate]
		for i := zerosStart; i < s.duplexRate; i++ {
			s.buf[i] = 0
		}
		s.buf[s.duplexRate-1] ^= 0x80
		xorState(&s.a, s.buf)
	} else if len(s.buf) != 0 {
		// otherwise we just pad with 0s for xorState to work
		zerosStart := len(s.buf) // rate = [0--end_of_buffer/zeroStart---duplexRate]
		s.buf = s.storage[:s.duplexRate]
		for i := zerosStart; i < s.duplexRate; i++ {
			s.buf[i] = 0
		}
		xorState(&s.a, s.buf)
	}

	// run the permutation
	keccakF1600(&s.a, 24)

	// reset the buffer and set posBegin to 0
	// (meaning that the current operation started on a previous block)
	s.buf = s.storage[:0]
	s.posBegin = 0
}

// duplex: the duplex call
func (s *Strobe) duplex(data []byte, cbefore, cafter, forceF bool) {

	// process data block by block
	for len(data) > 0 {

		todo := s.StrobeR - len(s.buf)
		if todo > len(data) {
			todo = len(data)
		}

		if cbefore {
			outState(s.a, s.tempStateBuf)
			for idx, state := range s.tempStateBuf[len(s.buf) : len(s.buf)+todo] {
				data[idx] ^= state
			}
		}

		// buffer what's to be XOR'ed (we XOR once during runF)
		s.buf = append(s.buf, data[:todo]...)

		if cafter {
			outState(s.a, s.tempStateBuf)
			for idx, state := range s.tempStateBuf[len(s.buf)-todo : len(s.buf)] {
				data[idx] ^= state
			}
		}

		// what's next for the loop?
		data = data[todo:]

		// If the duplex is full, time to XOR + padd + permutate.
		if len(s.buf) == s.StrobeR {
			s.runF()
		}

	}

	// sometimes we the next operation to start on a new block
	if forceF && len(s.buf) != 0 {
		s.runF()
	}

	return
}

// Operate runs an operation (see OperationMap for a list of operations).
// For operations that only require a length, provide the length via the
// length argument with an empty slice []byte{}. For other operations provide
// a zero length.
// Result is always retrieved through the return value. For boolean results,
// check that the first index is 0 for true, 1 for false.
func (s *Strobe) Operate(meta bool, operation string, dataConst []byte, length int, more bool) []byte {
	// operation is valid?
	var flags flag
	var ok bool
	if flags, ok = operationMap[operation]; !ok {
		panic("not a valid operation")
	}

	// operation is meta?
	if meta {
		flags |= flagM
	}

	// does the operation requires a length?
	var data []byte

	if (flags&(flagI|flagT) != (flagI | flagT)) && (flags&(flagI|flagA) != flagA) {

		if length == 0 {
			panic("A length should be set for this operation.")
		}

		data = bytes.Repeat([]byte{0}, length)

	} else {
		if length != 0 {
			panic("Output length must be zero except for PRF, send_MAC and RATCHET operations.")
		}

		data = make([]byte, len(dataConst))
		copy(data, dataConst)
	}

	// is this call the continuity of a previous call?
	if more {
		if flags != s.curFlags {
			panic("Flag should be the same when streaming operations.")
		}
	} else {
		s.beginOp(flags)
		s.curFlags = flags
	}

	// Operation
	cAfter := (flags & (flagC | flagI | flagT)) == (flagC | flagT)
	cBefore := (flags&flagC != 0) && (!cAfter)

	s.duplex(data, cBefore, cAfter, false)

	if (flags & (flagI | flagA)) == (flagI | flagA) {
		// Return data for the application
		return data
	} else if (flags & (flagI | flagT)) == flagT {
		// Return data for the transport.
		return data
	} else if (flags & (flagI | flagA | flagT)) == (flagI | flagT) {
		// Check MAC: all output bytes must be 0
		if more {
			panic("not supposed to check a MAC with the 'more' streaming option")
		}
		var failures byte
		for _, dataByte := range data {
			failures |= dataByte
		}
		return []byte{failures} // 0 if correct, 1 if not
	}

	// Operation has no output
	return nil
}

// beginOp: starts an operation
func (s *Strobe) beginOp(flags flag) {

	if flags&flagT != 0 {
		if s.I0 == iNone {
			s.I0 = role(flags & flagI)
		}
		flags ^= flag(s.I0)
	}

	oldBegin := s.posBegin
	s.posBegin = uint8(len(s.buf) + 1) // s.pos + 1
	forceF := (flags&(flagC|flagK) != 0)
	s.duplex([]byte{oldBegin, byte(flags)}, false, false, forceF)
}
