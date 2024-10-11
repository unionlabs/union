# Unencrypted Keys Generator

This tool is used to generate JSON file of unencrypted, base64 encoded, validator
signing and withdrawal keys. These keys can be fed into the Prysm validator
client for fast development startup times instead of using the Prysm keystore.

Usage:

```
bazel run //tools/unencrypted-keys-gen -- --num-keys 64 --output-json /path/to/output.json
```

Which will create 64 BLS private keys each for validator signing and withdrawals. 
These will then be output to an `output.json` file. Both arguments are required. 
The file can then be used to start the Prysm validator with the command:

```
bazel run //validator -- --unencrypted-keys /path/to/output.json
```