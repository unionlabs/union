# cosmos-exim

A small utility to benchmark export/import of Cosmos Hub IAVL stores. These stores can be downloaded e.g. from [chainlayer.io](https://www.chainlayer.io). Example usage:

```sh
$ go run benchmarks/cosmos-exim/main.go ../cosmoshub-3/data
Exporting cosmoshub database at version 870068

acc          : 67131 nodes (33566 leaves) in 676ms with size 3 MB
distribution : 66509 nodes (33255 leaves) in 804ms with size 3 MB
evidence     : 0 nodes (0 leaves) in 0s with size 0 MB
god          : 0 nodes (0 leaves) in 0s with size 0 MB
main         : 1 nodes (1 leaves) in 0s with size 0 MB
mint         : 1 nodes (1 leaves) in 0s with size 0 MB
params       : 59 nodes (30 leaves) in 0s with size 0 MB
slashing     : 1128139 nodes (564070 leaves) in 17.423s with size 41 MB
staking      : 44573 nodes (22287 leaves) in 433ms with size 3 MB
supply       : 1 nodes (1 leaves) in 0s with size 0 MB
upgrade      : 0 nodes (0 leaves) in 0s with size 0 MB

Exported 11 stores with 1306414 nodes (653211 leaves) in 19.336s with size 52 MB

Importing into new LevelDB stores

acc         : 67131 nodes (33566 leaves) in 259ms with size 3 MB
distribution: 66509 nodes (33255 leaves) in 238ms with size 3 MB
evidence    : 0 nodes (0 leaves) in 19ms with size 0 MB
god         : 0 nodes (0 leaves) in 40ms with size 0 MB
main        : 1 nodes (1 leaves) in 22ms with size 0 MB
mint        : 1 nodes (1 leaves) in 26ms with size 0 MB
params      : 59 nodes (30 leaves) in 26ms with size 0 MB
slashing    : 1128139 nodes (564070 leaves) in 5.213s with size 41 MB
staking     : 44573 nodes (22287 leaves) in 173ms with size 3 MB
supply      : 1 nodes (1 leaves) in 25ms with size 0 MB
upgrade     : 0 nodes (0 leaves) in 26ms with size 0 MB

Imported 11 stores with 1306414 nodes (653211 leaves) in 6.067s with size 52 MB
```