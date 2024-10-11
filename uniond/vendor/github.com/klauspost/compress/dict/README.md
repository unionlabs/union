# Dictionary builder

This is an *experimental* dictionary builder for Zstandard, S2, LZ4, deflate and more.

This diverges from the Zstandard dictionary builder, and may have some failure scenarios for very small or uniform inputs.

Dictionaries returned should all be valid, but if very little data is supplied, it may not be able to generate a dictionary.

With a large, diverse sample set, it will generate a dictionary that can compete with the Zstandard dictionary builder,
but for very similar data it will not be able to generate a dictionary that is as good.

Feedback is welcome.

## Usage

First of all a collection of *samples* must be collected.

These samples should be representative of the input data and should not contain any complete duplicates.

Only the *beginning* of the samples is important, the rest can be truncated. 
Beyond something like 64KB the input is not important anymore.  
The commandline tool can do this truncation for you. 

## Command line

To install the command line tool run:

```
$ go install github.com/klauspost/compress/dict/cmd/builddict@latest
```

Collect the samples in a directory, for example `samples/`.

Then run the command line tool. Basic usage is just to pass the directory with the samples:

```
$ builddict samples/
```

This will build a Zstandard dictionary and write it to `dictionary.bin` in the current folder.

The dictionary can be used with the Zstandard command line tool:

```
$ zstd -D dictionary.bin input
```

### Options

The command line tool has a few options:

- `-format`. Output type. "zstd" "s2" or "raw". Default "zstd".

Output a dictionary in Zstandard format, S2 format or raw bytes.
The raw bytes can be used with Deflate, LZ4, etc.

- `-hash` Hash bytes match length. Minimum match length. Must be 4-8 (inclusive) Default 6.

The hash bytes are used to define the shortest matches to look for.
Shorter matches can generate a more fractured dictionary with less compression, but can for certain inputs be better.
Usually lengths around 6-8 are best.

- `-len` Specify custom output size. Default 114688.
- `-max` Max input length to index per input file. Default 32768. All inputs are truncated to this.
- `-o` Output name. Default `dictionary.bin`.
- `-q`    Do not print progress
- `-dictID` zstd dictionary ID. 0 will be random. Default 0.
- `-zcompat` Generate dictionary compatible with zstd 1.5.5 and older. Default false.
- `-zlevel` Zstandard compression level.

The Zstandard compression level to use when compressing the samples.
The dictionary will be built using the specified encoder level, 
which will reflect speed and make the dictionary tailored for that level.
Default will use level 4 (best).

Valid values are 1-4, where 1 = fastest, 2 = default, 3 = better, 4 = best.

## Library

The `github.com/klaupost/compress/dict` package can be used to build dictionaries in code.
The caller must supply a collection of (pre-truncated) samples, and the options to use.
The options largely correspond to the command line options.

```Go
package main

import (
	"github.com/klaupost/compress/dict"
	"github.com/klauspost/compress/zstd"
)

func main() {
	var samples [][]byte

	// ... Fill samples with representative data.

	dict, err := dict.BuildZstdDict(samples, dict.Options{
		HashLen:     6,
		MaxDictSize: 114688,
		ZstdDictID:  0, // Random
		ZstdCompat:  false,
		ZstdLevel:   zstd.SpeedBestCompression,
	})
	// ... Handle error, etc.
}
```

There are similar functions for S2 and raw dictionaries (`BuildS2Dict` and `BuildRawDict`).
