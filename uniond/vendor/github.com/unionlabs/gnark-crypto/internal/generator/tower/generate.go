package tower

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/consensys/bavard"
	"github.com/consensys/gnark-crypto/internal/generator/config"
	"github.com/consensys/gnark-crypto/internal/generator/tower/asm/amd64"
)

// Generate generates a tower 2->6->12 over fp
func Generate(conf config.Curve, baseDir string, bgen *bavard.BatchGenerator) error {
	if conf.Equal(config.BW6_756) || conf.Equal(config.BW6_761) || conf.Equal(config.BW6_633) || conf.Equal(config.BLS24_315) || conf.Equal(config.BLS24_317) {
		return nil
	}

	entries := []bavard.Entry{
		{File: filepath.Join(baseDir, "e2_amd64.go"), Templates: []string{"amd64.fq2.go.tmpl"}},
		{File: filepath.Join(baseDir, "e2_fallback.go"), Templates: []string{"fallback.fq2.go.tmpl"}, BuildTag: "!amd64"},
		{File: filepath.Join(baseDir, "asm.go"), Templates: []string{"asm.go.tmpl"}, BuildTag: "!noadx"},
		{File: filepath.Join(baseDir, "asm_noadx.go"), Templates: []string{"asm_noadx.go.tmpl"}, BuildTag: "noadx"},
	}

	if err := bgen.Generate(conf, "fptower", "./tower/template/fq12over6over2", entries...); err != nil {
		return err
	}

	towerConfs := []towerConf{
		{
			Curve:           &conf,
			RecursionDegree: 2,
			TotalDegree:     2,
			BaseName:        "fp.Element",
			BaseElementName: "A",
		},
		{
			Curve:           &conf,
			RecursionDegree: 3,
			TotalDegree:     6,
			BaseName:        "E2",
			BaseElementName: "B",
		},
		{
			Curve:           &conf,
			RecursionDegree: 2,
			TotalDegree:     12,
			BaseName:        "E6",
			BaseElementName: "C",
		},
	}

	for _, towerConf := range towerConfs {

		entries = []bavard.Entry{
			{
				File:      filepath.Join(baseDir, fmt.Sprintf("e%d.go", towerConf.TotalDegree)),
				Templates: []string{fmt.Sprintf("fq%d.go.tmpl", towerConf.TotalDegree), "base.go.tmpl"},
			},
			{
				File:      filepath.Join(baseDir, fmt.Sprintf("e%d_test.go", towerConf.TotalDegree)),
				Templates: []string{fmt.Sprintf("tests/fq%d.go.tmpl", towerConf.TotalDegree), "tests/base.go.tmpl"},
			},
		}

		if err := bgen.Generate(towerConf, "fptower", "./tower/template/fq12over6over2", entries...); err != nil {
			return err
		}
	}

	{
		// fq2 assembly
		fName := filepath.Join(baseDir, "e2_amd64.s")
		f, err := os.Create(fName)
		if err != nil {
			return err
		}

		Fq2Amd64 := amd64.NewFq2Amd64(f, conf.Fp, conf)
		if err := Fq2Amd64.Generate(true); err != nil {
			_ = f.Close()
			return err
		}
		_ = f.Close()

	}

	if conf.Equal(config.BN254) || conf.Equal(config.BLS12_381) {
		{
			// fq2 assembly
			fName := filepath.Join(baseDir, "e2_adx_amd64.s")
			os.Remove(fName)
		}
	}

	return nil

}

type towerConf struct {
	Curve           *config.Curve
	RecursionDegree int
	TotalDegree     int
	BaseName        string
	BaseElementName string
}
