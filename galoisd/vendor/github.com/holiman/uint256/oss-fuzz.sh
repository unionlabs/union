#!/bin/bash -eu
compile_go_fuzzer github.com/holiman/uint256  Fuzz      uint256Fuzz
compile_go_fuzzer github.com/holiman/uint256  FuzzSetString      uint256FuzzSetString
