package eddsa

import (
	"path/filepath"

	"github.com/consensys/bavard"
	"github.com/consensys/gnark-crypto/internal/generator/config"
)

func Generate(conf config.TwistedEdwardsCurve, baseDir string, bgen *bavard.BatchGenerator) error {
	// eddsa
	conf.Package = "eddsa"
	baseDir = filepath.Join(baseDir, conf.Package)

	entries := []bavard.Entry{
		{File: filepath.Join(baseDir, "doc.go"), Templates: []string{"doc.go.tmpl"}},
		{File: filepath.Join(baseDir, "eddsa.go"), Templates: []string{"eddsa.go.tmpl"}},
		{File: filepath.Join(baseDir, "eddsa_test.go"), Templates: []string{"eddsa.test.go.tmpl"}},
		{File: filepath.Join(baseDir, "marshal.go"), Templates: []string{"marshal.go.tmpl"}},
	}
	return bgen.Generate(conf, conf.Package, "./edwards/eddsa/template", entries...)

}
