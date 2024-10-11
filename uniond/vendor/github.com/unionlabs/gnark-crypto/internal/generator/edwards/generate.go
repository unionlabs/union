package edwards

import (
	"path/filepath"

	"github.com/consensys/bavard"
	"github.com/consensys/gnark-crypto/internal/generator/config"
)

func Generate(conf config.TwistedEdwardsCurve, baseDir string, bgen *bavard.BatchGenerator) error {
	entries := []bavard.Entry{
		{File: filepath.Join(baseDir, "point.go"), Templates: []string{"point.go.tmpl"}},
		{File: filepath.Join(baseDir, "point_test.go"), Templates: []string{"tests/point.go.tmpl"}},
		{File: filepath.Join(baseDir, "doc.go"), Templates: []string{"doc.go.tmpl"}},
		{File: filepath.Join(baseDir, "curve.go"), Templates: []string{"curve.go.tmpl"}},
	}

	return bgen.Generate(conf, conf.Package, "./edwards/template", entries...)
}
