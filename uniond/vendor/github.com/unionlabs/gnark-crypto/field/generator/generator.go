package generator

import (
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"text/template"

	"github.com/consensys/bavard"
	"github.com/consensys/gnark-crypto/field/generator/asm/amd64"
	"github.com/consensys/gnark-crypto/field/generator/config"
	"github.com/consensys/gnark-crypto/field/generator/internal/addchain"
	"github.com/consensys/gnark-crypto/field/generator/internal/templates/element"
)

// GenerateFF will generate go (and .s) files in outputDir for modulus (in base 10)
//
// Example usage
//
//	fp, _ = config.NewField("fp", "Element", fpModulus")
//	generator.GenerateFF(fp, filepath.Join(baseDir, "fp"))
func GenerateFF(F *config.FieldConfig, outputDir string) error {
	// source file templates
	sourceFiles := []string{
		element.Base,
		element.Reduce,
		element.Exp,
		element.Conv,
		element.MulDoc,
		element.MulCIOS,
		element.MulNoCarry,
		element.Sqrt,
		element.Inverse,
		element.BigNum,
	}

	// test file templates
	testFiles := []string{
		element.MulCIOS,
		element.MulNoCarry,
		element.Reduce,
		element.Test,
		element.InverseTests,
	}
	// output files
	eName := strings.ToLower(F.ElementName)

	pathSrc := filepath.Join(outputDir, eName+".go")
	pathSrcVector := filepath.Join(outputDir, "vector.go")
	pathSrcFixedExp := filepath.Join(outputDir, eName+"_exp.go")
	pathSrcArith := filepath.Join(outputDir, "arith.go")
	pathTest := filepath.Join(outputDir, eName+"_test.go")
	pathTestVector := filepath.Join(outputDir, "vector_test.go")

	// remove old format generated files
	oldFiles := []string{"_mul.go", "_mul_amd64.go",
		"_square.go", "_square_amd64.go", "_ops_decl.go", "_square_amd64.s",
		"_mul_amd64.s",
		"_mul_arm64.s",
		"_mul_arm64.go",
		"_ops_amd64.s",
		"_ops_noasm.go",
		"_mul_adx_amd64.s",
		"_ops_amd64.go",
		"_fuzz.go",
	}

	for _, of := range oldFiles {
		_ = os.Remove(filepath.Join(outputDir, eName+of))
	}
	_ = os.Remove(filepath.Join(outputDir, "asm.go"))
	_ = os.Remove(filepath.Join(outputDir, "asm_noadx.go"))

	funcs := template.FuncMap{}
	if F.UseAddChain {
		for _, f := range addchain.Functions {
			funcs[f.Name] = f.Func
		}
	}

	funcs["shorten"] = shorten
	funcs["ltu64"] = func(a, b uint64) bool {
		return a < b
	}

	bavardOpts := []func(*bavard.Bavard) error{
		bavard.Apache2("ConsenSys Software Inc.", 2020),
		bavard.Package(F.PackageName),
		bavard.GeneratedBy("consensys/gnark-crypto"),
		bavard.Funcs(funcs),
	}

	// generate source file
	if err := bavard.GenerateFromString(pathSrc, sourceFiles, F, bavardOpts...); err != nil {
		return err
	}

	// generate vector
	if err := bavard.GenerateFromString(pathSrcVector, []string{element.Vector}, F, bavardOpts...); err != nil {
		return err
	}

	// generate arithmetics source file
	if err := bavard.GenerateFromString(pathSrcArith, []string{element.Arith}, F, bavardOpts...); err != nil {
		return err
	}

	// generate fixed exp source file
	if F.UseAddChain {
		if err := bavard.GenerateFromString(pathSrcFixedExp, []string{element.FixedExp}, F, bavardOpts...); err != nil {
			return err
		}
	}

	// generate test file
	if err := bavard.GenerateFromString(pathTest, testFiles, F, bavardOpts...); err != nil {
		return err
	}

	if err := bavard.GenerateFromString(pathTestVector, []string{element.TestVector}, F, bavardOpts...); err != nil {
		return err
	}

	// if we generate assembly code
	if F.ASM {
		// generate ops.s
		{
			pathSrc := filepath.Join(outputDir, eName+"_ops_amd64.s")
			fmt.Println("generating", pathSrc)
			f, err := os.Create(pathSrc)
			if err != nil {
				return err
			}

			_, _ = io.WriteString(f, "// +build !purego\n")

			if err := amd64.Generate(f, F); err != nil {
				_ = f.Close()
				return err
			}
			_ = f.Close()

			// run asmfmt
			// run go fmt on whole directory
			cmd := exec.Command("asmfmt", "-w", pathSrc)
			cmd.Stdout = os.Stdout
			cmd.Stderr = os.Stderr
			if err := cmd.Run(); err != nil {
				return err
			}
		}

		{
			pathSrc := filepath.Join(outputDir, eName+"_mul_amd64.s")
			fmt.Println("generating", pathSrc)
			f, err := os.Create(pathSrc)
			if err != nil {
				return err
			}

			_, _ = io.WriteString(f, "// +build !purego\n")

			if err := amd64.GenerateMul(f, F); err != nil {
				_ = f.Close()
				return err
			}
			_ = f.Close()

			// run asmfmt
			// run go fmt on whole directory
			cmd := exec.Command("asmfmt", "-w", pathSrc)
			cmd.Stdout = os.Stdout
			cmd.Stderr = os.Stderr
			if err := cmd.Run(); err != nil {
				return err
			}
		}

	}

	if F.ASM {
		// generate ops_amd64.go
		src := []string{
			element.MulDoc,
			element.OpsAMD64,
		}
		pathSrc := filepath.Join(outputDir, eName+"_ops_amd64.go")
		bavardOptsCpy := make([]func(*bavard.Bavard) error, len(bavardOpts))
		copy(bavardOptsCpy, bavardOpts)
		if F.ASM {
			bavardOptsCpy = append(bavardOptsCpy, bavard.BuildTag("!purego"))
		}
		if err := bavard.GenerateFromString(pathSrc, src, F, bavardOptsCpy...); err != nil {
			return err
		}
	}

	{
		// generate ops.go
		src := []string{
			element.OpsNoAsm,
			element.MulCIOS,
			element.MulNoCarry,
			element.Reduce,
			element.MulDoc,
		}
		pathSrc := filepath.Join(outputDir, eName+"_ops_purego.go")
		bavardOptsCpy := make([]func(*bavard.Bavard) error, len(bavardOpts))
		copy(bavardOptsCpy, bavardOpts)
		if F.ASM {
			bavardOptsCpy = append(bavardOptsCpy, bavard.BuildTag("!amd64 purego"))
		}
		if err := bavard.GenerateFromString(pathSrc, src, F, bavardOptsCpy...); err != nil {
			return err
		}
	}

	{
		// generate doc.go
		src := []string{
			element.Doc,
		}
		pathSrc := filepath.Join(outputDir, "doc.go")
		if err := bavard.GenerateFromString(pathSrc, src, F, bavardOpts...); err != nil {
			return err
		}
	}

	if F.ASM {
		// generate asm.go and asm_noadx.go
		src := []string{
			element.Asm,
		}
		pathSrc := filepath.Join(outputDir, "asm.go")
		bavardOptsCpy := make([]func(*bavard.Bavard) error, len(bavardOpts))
		copy(bavardOptsCpy, bavardOpts)
		bavardOptsCpy = append(bavardOptsCpy, bavard.BuildTag("!noadx"))
		if err := bavard.GenerateFromString(pathSrc, src, F, bavardOptsCpy...); err != nil {
			return err
		}
	}
	if F.ASM {
		// generate asm.go and asm_noadx.go
		src := []string{
			element.AsmNoAdx,
		}
		pathSrc := filepath.Join(outputDir, "asm_noadx.go")
		bavardOptsCpy := make([]func(*bavard.Bavard) error, len(bavardOpts))
		copy(bavardOptsCpy, bavardOpts)
		bavardOptsCpy = append(bavardOptsCpy, bavard.BuildTag("noadx"))
		if err := bavard.GenerateFromString(pathSrc, src, F, bavardOptsCpy...); err != nil {
			return err
		}
	}

	// run go fmt on whole directory
	cmd := exec.Command("gofmt", "-s", "-w", outputDir)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		return err
	}

	return nil
}

func shorten(input string) string {
	const maxLen = 15
	if len(input) > maxLen {
		return input[:6] + "..." + input[len(input)-6:]
	}
	return input
}
