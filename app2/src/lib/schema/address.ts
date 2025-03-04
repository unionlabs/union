import { Schema } from "effect"
import { Hex, HexChecksum } from "$lib/schema/hex"
import { Bech32 } from "$lib/schema/bech32"

// For Reference, see: https://docs.union.build/concepts/address-types/
// We always store bytes arrays as hex-encoded strings
export const AddressCanonicalBytes = Hex.pipe(Schema.brand("CanonicalBytes"))

// Cosmos Address Types
export const AddressCosmosCanonical = AddressCanonicalBytes.pipe(
  Schema.brand("AddressCosmosCanonical")
)
export const AddressCosmosDisplay = Bech32.pipe(Schema.brand("AddressCosmosDisplay"))
export const AddressCosmosZkgm = Hex.pipe(Schema.brand("AddressCosmosZkgm")) // Hex<Bech32<Hrp, Cosmos.Canonical>>

// EVM Address Types
export const AddressEvmCanonical = AddressCanonicalBytes.pipe(Schema.brand("AddressEvmCanonical"))
export const AddressEvmDisplay = HexChecksum.pipe(Schema.brand("AddressEvmDisplay"))
export const AddressEvmZkgm = AddressEvmCanonical

// Aptos Address Types
export const AddressAptosCanonical = AddressCanonicalBytes.pipe(
  Schema.brand("AddressAptosCanonical")
)
export const AddressAptosDisplay = AddressAptosCanonical
export const AddressAptosZkgm = AddressAptosCanonical
