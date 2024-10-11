package fri

import (
	"path/filepath"

	"github.com/consensys/bavard"
	"github.com/consensys/gnark-crypto/internal/generator/config"
)

func Generate(conf config.Curve, baseDir string, bgen *bavard.BatchGenerator) error {

	// fri commitment scheme
	conf.Package = "fri"
	entries := []bavard.Entry{
		{File: filepath.Join(baseDir, "doc.go"), Templates: []string{"doc.go.tmpl"}},
		{File: filepath.Join(baseDir, "fri.go"), Templates: []string{"fri.go.tmpl"}},
		{File: filepath.Join(baseDir, "fri_test.go"), Templates: []string{"fri.test.go.tmpl"}},
	}
	return bgen.Generate(conf, conf.Package, "./fri/template/", entries...)

}
