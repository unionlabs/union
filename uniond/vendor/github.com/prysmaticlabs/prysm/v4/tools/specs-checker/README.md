# Specs checker tool

This simple tool helps downloading and parsing [Ethereum specs](https://github.com/ethereum/consensus-specs/tree/dev/specs), 
to be later used for making sure that our reference comments match specs definitions precisely.

### Updating the reference specs
See `main.go` for a list of files to be downloaded, currently:
```golang
var specDirs = map[string][]string{
	"specs/phase0": {
		"beacon-chain.md",
		"fork-choice.md",
		"validator.md",
		"weak-subjectivity.md",
	},
	"ssz": {
		"merkle-proofs.md",
	},
}
```

To download/update specs:
```bash
bazel run //tools/specs-checker download -- --dir=$PWD/tools/specs-checker/data
```

This will pull the files defined in `specDirs`, parse them (extract Python code snippets, discarding any other text), 
and save them to the folder from which `bazel run //tools/specs-checker check` will be able to embed.

### Checking against the reference specs

To check whether reference comments have the matching version of Python specs:
```bash
bazel run //tools/specs-checker check -- --dir $PWD/beacon-chain
bazel run //tools/specs-checker check -- --dir $PWD/validator
bazel run //tools/specs-checker check -- --dir $PWD/shared
```
Or, to check the whole project:
```bash
bazel run //tools/specs-checker check -- --dir $PWD
```
