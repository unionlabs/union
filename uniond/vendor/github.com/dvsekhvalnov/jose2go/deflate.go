package jose

import (
	"bytes"
	"compress/flate"
	"io/ioutil"
)

func init() {
	RegisterJwc(new(Deflate))
}

// Deflate compression algorithm implementation
type Deflate struct {}

func (alg *Deflate) Name() string {
	return DEF
}

func (alg *Deflate) Compress(plainText []byte) []byte {
	var buf bytes.Buffer
	deflate,_ := flate.NewWriter(&buf, 8) //level=DEFLATED
	
	deflate.Write(plainText)
	deflate.Close()
	
	return buf.Bytes()
}

func (alg *Deflate) Decompress(compressedText []byte) []byte {	
	
	enflated,_ := ioutil.ReadAll(
					flate.NewReader(
						bytes.NewReader(compressedText)))
	
	return enflated
}


