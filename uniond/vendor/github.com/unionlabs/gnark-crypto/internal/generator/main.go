package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"sync"

	"github.com/consensys/bavard"
	"github.com/consensys/gnark-crypto/field/generator"
	field "github.com/consensys/gnark-crypto/field/generator/config"
	"github.com/consensys/gnark-crypto/internal/generator/config"
	"github.com/consensys/gnark-crypto/internal/generator/crypto/hash/mimc"
	"github.com/consensys/gnark-crypto/internal/generator/ecc"
	"github.com/consensys/gnark-crypto/internal/generator/ecdsa"
	"github.com/consensys/gnark-crypto/internal/generator/edwards"
	"github.com/consensys/gnark-crypto/internal/generator/edwards/eddsa"
	"github.com/consensys/gnark-crypto/internal/generator/fft"
	fri "github.com/consensys/gnark-crypto/internal/generator/fri/template"
	"github.com/consensys/gnark-crypto/internal/generator/gkr"
	"github.com/consensys/gnark-crypto/internal/generator/iop"
	"github.com/consensys/gnark-crypto/internal/generator/kzg"
	"github.com/consensys/gnark-crypto/internal/generator/pairing"
	"github.com/consensys/gnark-crypto/internal/generator/pedersen"
	"github.com/consensys/gnark-crypto/internal/generator/permutation"
	"github.com/consensys/gnark-crypto/internal/generator/plookup"
	"github.com/consensys/gnark-crypto/internal/generator/polynomial"
	"github.com/consensys/gnark-crypto/internal/generator/sis"
	"github.com/consensys/gnark-crypto/internal/generator/sumcheck"
	"github.com/consensys/gnark-crypto/internal/generator/test_vector_utils"
	"github.com/consensys/gnark-crypto/internal/generator/tower"
)

const (
	copyrightHolder = "Consensys Software Inc."
	copyrightYear   = 2020
	baseDir         = "../../"
)

var bgen = bavard.NewBatchGenerator(copyrightHolder, copyrightYear, "consensys/gnark-crypto")

//go:generate go run main.go
func main() {
	var wg sync.WaitGroup

	for _, conf := range config.Curves {
		wg.Add(1)
		// for each curve, generate the needed files
		go func(conf config.Curve) {
			defer wg.Done()
			var err error

			curveDir := filepath.Join(baseDir, "ecc", conf.Name)

			// generate base field
			conf.Fp, err = field.NewFieldConfig("fp", "Element", conf.FpModulus, true)
			assertNoError(err)

			conf.Fr, err = field.NewFieldConfig("fr", "Element", conf.FrModulus, !conf.Equal(config.STARK_CURVE))
			assertNoError(err)

			conf.FpUnusedBits = 64 - (conf.Fp.NbBits % 64)

			assertNoError(generator.GenerateFF(conf.Fr, filepath.Join(curveDir, "fr")))
			assertNoError(generator.GenerateFF(conf.Fp, filepath.Join(curveDir, "fp")))

			// generate ecdsa
			assertNoError(ecdsa.Generate(conf, curveDir, bgen))

			if conf.Equal(config.STARK_CURVE) {
				return // TODO @yelhousni
			}

			// generate G1, G2, multiExp, ...
			assertNoError(ecc.Generate(conf, curveDir, bgen))

			if conf.Equal(config.SECP256K1) {
				return
			}

			// generate tower of extension
			assertNoError(tower.Generate(conf, filepath.Join(curveDir, "internal", "fptower"), bgen))

			// generate pairing tests
			assertNoError(pairing.Generate(conf, curveDir, bgen))

			// generate fri on fr
			assertNoError(fri.Generate(conf, filepath.Join(curveDir, "fr", "fri"), bgen))

			// generate fft on fr
			assertNoError(fft.Generate(conf, filepath.Join(curveDir, "fr", "fft"), bgen))

			if conf.Equal(config.BN254) {
				assertNoError(sis.Generate(conf, filepath.Join(curveDir, "fr", "sis"), bgen))
			}

			// generate kzg on fr
			assertNoError(kzg.Generate(conf, filepath.Join(curveDir, "kzg"), bgen))

			// generate pedersen on fr
			assertNoError(pedersen.Generate(conf, filepath.Join(curveDir, "fr", "pedersen"), bgen))

			// generate plookup on fr
			assertNoError(plookup.Generate(conf, filepath.Join(curveDir, "fr", "plookup"), bgen))

			// generate permutation on fr
			assertNoError(permutation.Generate(conf, filepath.Join(curveDir, "fr", "permutation"), bgen))

			// generate mimc on fr
			assertNoError(mimc.Generate(conf, filepath.Join(curveDir, "fr", "mimc"), bgen))

			frInfo := config.FieldDependency{
				FieldPackagePath: "github.com/consensys/gnark-crypto/ecc/" + conf.Name + "/fr",
				FieldPackageName: "fr",
				ElementType:      "fr.Element",
			}

			// generate polynomial on fr
			assertNoError(polynomial.Generate(frInfo, filepath.Join(curveDir, "fr", "polynomial"), true, bgen))

			// generate sumcheck on fr
			assertNoError(sumcheck.Generate(frInfo, filepath.Join(curveDir, "fr", "sumcheck"), bgen))

			// generate gkr on fr
			assertNoError(gkr.Generate(gkr.Config{
				FieldDependency:         frInfo,
				GenerateTests:           true,
				TestVectorsRelativePath: "../../../../internal/generator/gkr/test_vectors",
			}, filepath.Join(curveDir, "fr", "gkr"), bgen))

			// generate test vector utils on fr
			assertNoError(test_vector_utils.Generate(test_vector_utils.Config{
				FieldDependency:             frInfo,
				RandomizeMissingHashEntries: false,
			}, filepath.Join(curveDir, "fr", "test_vector_utils"), bgen))

			// generate eddsa on companion curves
			assertNoError(fri.Generate(conf, filepath.Join(curveDir, "fr", "fri"), bgen))

			// generate sumcheck on fr
			assertNoError(sumcheck.Generate(frInfo, filepath.Join(curveDir, "fr", "sumcheck"), bgen))

			// generate gkr on fr
			assertNoError(gkr.Generate(gkr.Config{
				FieldDependency:         frInfo,
				GenerateTests:           true,
				TestVectorsRelativePath: "../../../../internal/generator/gkr/test_vectors",
			}, filepath.Join(curveDir, "fr", "gkr"), bgen))

			// generate test vector utils on fr
			assertNoError(test_vector_utils.Generate(test_vector_utils.Config{
				FieldDependency:             frInfo,
				RandomizeMissingHashEntries: false,
			}, filepath.Join(curveDir, "fr", "test_vector_utils"), bgen))

			// generate iop functions
			assertNoError(iop.Generate(conf, filepath.Join(curveDir, "fr", "iop"), bgen))

		}(conf)

	}

	for _, conf := range config.TwistedEdwardsCurves {
		wg.Add(1)

		go func(conf config.TwistedEdwardsCurve) {
			defer wg.Done()

			curveDir := filepath.Join(baseDir, "ecc", conf.Name, conf.Package)
			// generate twisted edwards companion curves
			assertNoError(edwards.Generate(conf, curveDir, bgen))

			// generate eddsa on companion curves
			assertNoError(eddsa.Generate(conf, curveDir, bgen))
		}(conf)

	}

	wg.Add(1)
	go func() {
		defer wg.Done()
		assertNoError(test_vector_utils.GenerateRationals(bgen))
	}()
	wg.Wait()

	// format the whole directory

	cmd := exec.Command("gofmt", "-s", "-w", baseDir)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	assertNoError(cmd.Run())

	cmd = exec.Command("asmfmt", "-w", baseDir)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	assertNoError(cmd.Run())

	//mathfmt doesn't accept directories. TODO: PR pending
	/*cmd = exec.Command("mathfmt", "-w", baseDir)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	assertNoError(cmd.Run())*/

	wg.Add(2)

	go func() {
		// generate test vectors for sumcheck
		cmd := exec.Command("go", "run", "./sumcheck/test_vectors")
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		assertNoError(cmd.Run())
		wg.Done()
	}()

	go func() {
		// generate test vectors for gkr
		cmd := exec.Command("go", "run", "./gkr/test_vectors")
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		assertNoError(cmd.Run())
		wg.Done()
	}()

	wg.Wait()
}

func assertNoError(err error) {
	if err != nil {
		fmt.Printf("\n%s\n", err.Error())
		os.Exit(-1)
	}
}
