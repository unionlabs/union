package merlin

import (
	"encoding/binary"

	"github.com/mimoo/StrobeGo/strobe"
)

const (
	merlinProtocolLabel  = "Merlin v1.0"
	domainSeparatorLabel = "dom-sep"
)

type Transcript struct {
	s strobe.Strobe
}

func NewTranscript(appLabel string) *Transcript {
	t := Transcript{
		s: strobe.InitStrobe(merlinProtocolLabel, 128),
	}

	t.AppendMessage([]byte(domainSeparatorLabel), []byte(appLabel))
	return &t
}

// Append adds the message to the transcript with the supplied label.
func (t *Transcript) AppendMessage(label, message []byte) {
	// AD[label || le32(len(message))](message)

	sizeBuffer := make([]byte, 4)
	binary.LittleEndian.PutUint32(sizeBuffer[0:], uint32(len(message)))

	// The StrobeGo API does not support continuation operations,
	// so we have to pass the label and length as a single buffer.
	// Otherwise it will record two meta-AD operations instead of one.
	labelSize := append(label, sizeBuffer...)
	t.s.AD(true, labelSize)

	t.s.AD(false, message)
}

// ExtractBytes returns a buffer filled with the verifier's challenge bytes.
// The label parameter is metadata about the challenge, and is also appended to
// the transcript. See the Transcript Protocols section of the Merlin website
// for details on labels.
func (t *Transcript) ExtractBytes(label []byte, outLen int) []byte {
	sizeBuffer := make([]byte, 4)
	binary.LittleEndian.PutUint32(sizeBuffer[0:], uint32(outLen))

	// The StrobeGo API does not support continuation operations,
	// so we have to pass the label and length as a single buffer.
	// Otherwise it will record two meta-AD operations instead of one.
	labelSize := append(label, sizeBuffer...)
	t.s.AD(true, labelSize)

	// A PRF call directly to the output buffer (in the style of an append API)
	// would be better, but our underlying STROBE library forces an allocation
	// here.
	outBytes := t.s.PRF(outLen)
	return outBytes
}
