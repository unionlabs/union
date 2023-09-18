#!/bin/bash

set -euo pipefail

export FUZZ_ROOT="github.com/cosmos/cosmos-sdk"

build_go_fuzzer() {
	local function="$1"
	local fuzzer="$2"

	go run github.com/orijtech/otils/corpus2ossfuzz@latest -o "$OUT"/"$fuzzer"_seed_corpus.zip -corpus fuzz/tests/testdata/fuzz/"$function"
	compile_native_go_fuzzer "$FUZZ_ROOT"/fuzz/tests "$function" "$fuzzer"
}

go get github.com/AdamKorcz/go-118-fuzz-build/utils

build_go_fuzzer FuzzCryptoHDDerivePrivateKeyForPath fuzz_crypto_hd_deriveprivatekeyforpath
build_go_fuzzer FuzzCryptoHDNewParamsFromPath fuzz_crypto_hd_newparamsfrompath

build_go_fuzzer FuzzCryptoTypesCompactbitarrayMarshalUnmarshal fuzz_crypto_types_compactbitarray_marshalunmarshal

build_go_fuzzer FuzzStoreInternalProofsCreateNonmembershipProof fuzz_store_internal_proofs_createnonmembershipproof

build_go_fuzzer FuzzTendermintAminoDecodeTime fuzz_tendermint_amino_decodetime

build_go_fuzzer FuzzTypesParseCoin fuzz_types_parsecoin
build_go_fuzzer FuzzTypesParseDecCoin fuzz_types_parsedeccoin
build_go_fuzzer FuzzTypesParseTimeBytes fuzz_types_parsetimebytes
build_go_fuzzer FuzzTypesVerifyAddressFormat fuzz_types_verifyaddressformat
build_go_fuzzer FuzzTypesDecSetString fuzz_types_dec_setstring

build_go_fuzzer FuzzUnknownProto fuzz_unknownproto

build_go_fuzzer FuzzXBankTypesAddressFromBalancesStore fuzz_x_bank_types_addressfrombalancesstore
