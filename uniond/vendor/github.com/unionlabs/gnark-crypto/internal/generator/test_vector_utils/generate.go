package test_vector_utils

import (
	"github.com/consensys/bavard"
	"github.com/consensys/gnark-crypto/internal/generator/config"
	"github.com/consensys/gnark-crypto/internal/generator/gkr"
	"github.com/consensys/gnark-crypto/internal/generator/polynomial"
	"github.com/consensys/gnark-crypto/internal/generator/sumcheck"
	"path/filepath"
)

type Config struct {
	config.FieldDependency
	RandomizeMissingHashEntries bool
}

func GenerateRationals(bgen *bavard.BatchGenerator) error {
	gkrConf := gkr.Config{
		FieldDependency: config.FieldDependency{
			FieldPackagePath: "github.com/consensys/gnark-crypto/internal/generator/test_vector_utils/small_rational",
			FieldPackageName: "small_rational",
			ElementType:      "small_rational.SmallRational",
		},
		GenerateTests:           false,
		RetainTestCaseRawInfo:   true,
		TestVectorsRelativePath: "../../../gkr/test_vectors",
	}

	baseDir := "./test_vector_utils/small_rational/"
	if err := polynomial.Generate(gkrConf.FieldDependency, baseDir+"polynomial", false, bgen); err != nil {
		return err
	}
	if err := sumcheck.Generate(gkrConf.FieldDependency, baseDir+"sumcheck", bgen); err != nil {
		return err
	}
	if err := gkr.Generate(gkrConf, baseDir+"gkr", bgen); err != nil {
		return err
	}
	if err := Generate(Config{gkrConf.FieldDependency, true}, baseDir+"test_vector_utils", bgen); err != nil {
		return err
	}

	// generate gkr test vector generator for rationals
	gkrConf.OutsideGkrPackage = true
	return bgen.Generate(gkrConf, "main", "./gkr/template", bavard.Entry{
		File: filepath.Join("gkr", "test_vectors", "main.go"), Templates: []string{"gkr.test.vectors.gen.go.tmpl", "gkr.test.vectors.go.tmpl"},
	})

}

func Generate(conf Config, baseDir string, bgen *bavard.BatchGenerator) error {
	entry := bavard.Entry{
		File: filepath.Join(baseDir, "test_vector_utils.go"), Templates: []string{"test_vector_utils.go.tmpl"},
	}

	return bgen.Generate(conf, "test_vector_utils", "./test_vector_utils/template/", entry)
}
