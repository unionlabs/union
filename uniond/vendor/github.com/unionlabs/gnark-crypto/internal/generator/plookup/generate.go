package plookup

import (
	"path/filepath"

	"github.com/consensys/bavard"
	"github.com/consensys/gnark-crypto/internal/generator/config"
)

func Generate(conf config.Curve, baseDir string, bgen *bavard.BatchGenerator) error {

	// kzg commitment scheme
	conf.Package = "plookup"
	entries := []bavard.Entry{
		{File: filepath.Join(baseDir, "doc.go"), Templates: []string{"doc.go.tmpl"}},
		{File: filepath.Join(baseDir, "vector.go"), Templates: []string{"vector.go.tmpl"}},
		{File: filepath.Join(baseDir, "table.go"), Templates: []string{"table.go.tmpl"}},
		{File: filepath.Join(baseDir, "plookup_test.go"), Templates: []string{"plookup.test.go.tmpl"}},
	}
	return bgen.Generate(conf, conf.Package, "./plookup/template/", entries...)

}
