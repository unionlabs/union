package iop

import (
	"path/filepath"

	"github.com/consensys/bavard"
	"github.com/consensys/gnark-crypto/internal/generator/config"
)

func Generate(conf config.Curve, baseDir string, bgen *bavard.BatchGenerator) error {

	// fri commitment scheme
	conf.Package = "iop"
	entries := []bavard.Entry{
		{File: filepath.Join(baseDir, "doc.go"), Templates: []string{"doc.go.tmpl"}},
		{File: filepath.Join(baseDir, "polynomial.go"), Templates: []string{"polynomial.go.tmpl"}},
		{File: filepath.Join(baseDir, "polynomial_test.go"), Templates: []string{"polynomial.test.go.tmpl"}},

		{File: filepath.Join(baseDir, "ratios.go"), Templates: []string{"ratios.go.tmpl"}},
		{File: filepath.Join(baseDir, "ratios_test.go"), Templates: []string{"ratios.test.go.tmpl"}},

		{File: filepath.Join(baseDir, "quotient.go"), Templates: []string{"quotient.go.tmpl"}},
		{File: filepath.Join(baseDir, "quotient_test.go"), Templates: []string{"quotient.test.go.tmpl"}},

		{File: filepath.Join(baseDir, "expressions.go"), Templates: []string{"expressions.go.tmpl"}},
		{File: filepath.Join(baseDir, "expressions_test.go"), Templates: []string{"expressions.test.go.tmpl"}},

		{File: filepath.Join(baseDir, "utils.go"), Templates: []string{"utils.go.tmpl"}},
	}

	return bgen.Generate(conf, conf.Package, "./iop/template/", entries...)

}
