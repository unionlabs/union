package fft

import (
	"math/bits"
	"path/filepath"

	"github.com/consensys/bavard"
	"github.com/consensys/gnark-crypto/internal/generator/config"
)

func Generate(conf config.Curve, baseDir string, bgen *bavard.BatchGenerator) error {

	conf.Package = "fft"

	entries := []bavard.Entry{
		{File: filepath.Join(baseDir, "doc.go"), Templates: []string{"doc.go.tmpl"}},
		{File: filepath.Join(baseDir, "domain_test.go"), Templates: []string{"tests/domain.go.tmpl", "imports.go.tmpl"}},
		{File: filepath.Join(baseDir, "domain.go"), Templates: []string{"domain.go.tmpl", "imports.go.tmpl"}},
		{File: filepath.Join(baseDir, "fft_test.go"), Templates: []string{"tests/fft.go.tmpl", "imports.go.tmpl"}},
		{File: filepath.Join(baseDir, "bitreverse_test.go"), Templates: []string{"tests/bitreverse.go.tmpl", "imports.go.tmpl"}},
		{File: filepath.Join(baseDir, "fft.go"), Templates: []string{"fft.go.tmpl", "imports.go.tmpl"}},
		{File: filepath.Join(baseDir, "bitreverse.go"), Templates: []string{"bitreverse.go.tmpl", "imports.go.tmpl"}},
		{File: filepath.Join(baseDir, "options.go"), Templates: []string{"options.go.tmpl", "imports.go.tmpl"}},
	}

	funcs := make(map[string]interface{})
	funcs["bitReverse"] = func(n, i int64) uint64 {
		nn := uint64(64 - bits.TrailingZeros64(uint64(n)))
		r := make([]uint64, n)
		for i := 0; i < len(r); i++ {
			r[i] = uint64(i)
		}
		for i := 0; i < len(r); i++ {
			irev := bits.Reverse64(r[i]) >> nn
			if irev > uint64(i) {
				r[i], r[irev] = r[irev], r[i]
			}
		}
		return r[i]
	}
	funcs["reverseBits"] = func(x, n any) uint64 {
		return bits.Reverse64(anyToUint64(x)) >> anyToUint64(n)
	}
	funcs["shl"] = func(x, n any) uint64 {
		return anyToUint64(x) << anyToUint64(n)
	}
	funcs["logicalOr"] = func(x, y any) uint64 {
		return anyToUint64(x) | anyToUint64(y)
	}

	bavardOpts := []func(*bavard.Bavard) error{bavard.Funcs(funcs)}

	if err := bgen.GenerateWithOptions(conf, conf.Package, "./fft/template/", bavardOpts, entries...); err != nil {
		return err
	}

	// put the generator in the parent dir (fr)
	frDir := filepath.Dir(baseDir)
	entries = []bavard.Entry{
		{File: filepath.Join(frDir, "generator.go"), Templates: []string{"fr.generator.go.tmpl"}},
	}
	return bgen.GenerateWithOptions(conf, "fr", "./fft/template/", bavardOpts, entries...)
}

func anyToUint64(x any) uint64 {
	switch v := x.(type) {
	case int:
		return uint64(v)
	case int64:
		return uint64(v)
	case uint64:
		return v
	default:
		panic("unknown type")
	}
}
