# `gnark` zk-SNARK library

[![Twitter URL](https://img.shields.io/twitter/url/https/twitter.com/gnark_team.svg?style=social&label=Follow%20%40gnark_team)](https://twitter.com/gnark_team) [![License](https://img.shields.io/badge/license-Apache%202-blue)](LICENSE)
[![Go Report Card](https://goreportcard.com/badge/github.com/ConsenSys/gnark)](https://goreportcard.com/badge/github.com/ConsenSys/gnark)
[![PkgGoDev](https://pkg.go.dev/badge/mod/github.com/consensys/gnark)](https://pkg.go.dev/mod/github.com/consensys/gnark)
[![Documentation Status](https://readthedocs.com/projects/pegasys-gnark/badge/)][`gnark` User Documentation] [![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.5819104.svg)](https://doi.org/10.5281/zenodo.5819104)

`gnark` is a fast zk-SNARK library that offers a high-level API to design circuits. The library is open source and developed under the Apache 2.0 license


## Useful Links

* [`gnark` User Documentation]
* [`gnark` Playground]
* [`gnark` Issues]
* [`gnark` Benchmarks](https://docs.gnark.consensys.net/overview#gnark-is-fast) 🏁
* [`gnark-announce`] - Announcement list for new releases and security patches


## `gnark` Users

To get started with `gnark` and write your first circuit, follow [these instructions][`gnark` User Documentation].

Checkout the [online playground][`gnark` Playground] to compile circuits and visualize constraint systems.


## Warning

**`gnark` has been [partially audited](https://github.com/ConsenSys/gnark-crypto/blob/master/audit_oct2022.pdf) and is provided as-is, we make no guarantees or warranties to its safety and reliability. In particular, `gnark` makes no security guarantees such as constant time implementation or side-channel attack resistance.**

`gnark` and `gnark-crypto` packages are optimized for 64bits architectures (x86 `amd64`) and tested on Unix (Linux / macOS).

## Issues

`gnark` issues are tracked [in the GitHub issues tab][`gnark` Issues].

**To report a security bug, please refer to [`gnark` Security Policy](SECURITY.md).**

If you have any questions, queries or comments, [GitHub discussions] is the place to find us.

You can also get in touch directly: gnark@consensys.net

## Release Notes

[Release Notes](CHANGELOG.md)

## Proving schemes and curves

Refer to [Proving schemes and curves] for more details.

`gnark` support the following zk-SNARKs:

- [x] [Groth16](https://eprint.iacr.org/2016/260)
- [x] [PlonK](https://eprint.iacr.org/2019/953)

which can be instantiated with the following curves

- [x] BN254
- [x] BLS12-381
- [x] BLS12-377
- [x] BW6-761
- [x] BLS24-315
- [x] BW6-633
- [x] BLS24-317

### Example

Refer to the [`gnark` User Documentation]

Here is what `x**3 + x + 5 = y` looks like

```golang
package main

import (
	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
)

// CubicCircuit defines a simple circuit
// x**3 + x + 5 == y
type CubicCircuit struct {
	// struct tags on a variable is optional
	// default uses variable name and secret visibility.
	X frontend.Variable `gnark:"x"`
	Y frontend.Variable `gnark:",public"`
}

// Define declares the circuit constraints
// x**3 + x + 5 == y
func (circuit *CubicCircuit) Define(api frontend.API) error {
	x3 := api.Mul(circuit.X, circuit.X, circuit.X)
	api.AssertIsEqual(circuit.Y, api.Add(x3, circuit.X, 5))
	return nil
}

func main() {
	// compiles our circuit into a R1CS
	var circuit CubicCircuit
	ccs, _ := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &circuit)

	// groth16 zkSNARK: Setup
	pk, vk, _ := groth16.Setup(ccs)

	// witness definition
	assignment := CubicCircuit{X: 3, Y: 35}
	witness, _ := frontend.NewWitness(&assignment, ecc.BN254.ScalarField())
	publicWitness, _ := witness.Public()

	// groth16: Prove & Verify
	proof, _ := groth16.Prove(ccs, pk, witness)
	groth16.Verify(proof, vk, publicWitness)
}

```

### GPU Support

#### Icicle Library

The following schemes and curves support experimental use of Ingonyama's Icicle GPU library for low level zk-SNARK primitives such as MSM, NTT, and polynomial operations:

- [x] [Groth16](https://eprint.iacr.org/2016/260)

instantiated with the following curve(s)

- [x] BN254

To use GPUs, add the `icicle` buildtag to your build/run commands, e.g. `go run -tags=icicle main.go`.

You can then toggle on or off icicle acceleration by providing the `WithIcicleAcceleration` backend ProverOption:

```go
    // toggle on
    proofIci, err := groth16.Prove(ccs, pk, secretWitness, backend.WithIcicleAcceleration())
    
    // toggle off
    proof, err := groth16.Prove(ccs, pk, secretWitness)
```

For more information about prerequisites see the [Icicle repo](https://github.com/ingonyama-zk/icicle).

## Citing

If you use `gnark` in your research a citation would be appreciated.
Please use the following BibTeX to cite the most recent release.

```bib
@software{gnark-v0.9.0,
  author       = {Gautam Botrel and
                  Thomas Piellard and
                  Youssef El Housni and
                  Ivo Kubjas and
                  Arya Tabaie},
  title        = {ConsenSys/gnark: v0.9.0},
  month        = feb,
  year         = 2023,
  publisher    = {Zenodo},
  version      = {v0.9.0},
  doi          = {10.5281/zenodo.5819104},
  url          = {https://doi.org/10.5281/zenodo.5819104}
}
```

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our [code of conduct](CODE_OF_CONDUCT.md), and the process for submitting pull requests to us.

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/consensys/gnark/tags).

## License

This project is licensed under the Apache 2 License - see the [LICENSE](LICENSE) file for details

[`gnark` Issues]: https://github.com/consensys/gnark/issues
[`gnark` Playground]: https://play.gnark.io
[`gnark` User Documentation]: https://docs.gnark.consensys.net/
[GitHub discussions]: https://github.com/ConsenSys/gnark/discussions
[Proving schemes and curves]: https://docs.gnark.consensys.net/Concepts/schemes_curves
[`gnark-announce`]: https://groups.google.com/g/gnark-announce
[@gnark_team]: https://twitter.com/gnark_team
