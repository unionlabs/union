import * as S from "effect/Schema"
import { Hex, HexChecksum } from "./hex.js"
import { Bech32 } from "./bech32.js"

// For Reference, see: https://docs.union.build/concepts/address-types/
// We always store bytes arrays as hex-encoded strings
export const AddressCanonicalBytes = Hex.pipe(S.brand("CanonicalBytes"))
export type AddressCanonicalBytes = typeof AddressCanonicalBytes.Type

// Cosmos Address Types
export const AddressCosmosCanonical = AddressCanonicalBytes.pipe(
  S.brand("AddressCosmosCanonical")
)
export type AddressCosmosCanonical = typeof AddressCosmosCanonical.Type

export const AddressCosmosDisplay = Bech32.pipe(S.brand("AddressCosmosDisplay"))
export type AddressCosmosDisplay = typeof AddressCosmosDisplay.Type

export const AddressCosmosZkgm = Hex.pipe(S.brand("AddressCosmosZkgm")) // Hex<Bech32<Hrp, Cosmos.Canonical>>

// Evm Address Types
export const AddressEvmCanonical = AddressCanonicalBytes.pipe(S.brand("AddressEvmCanonical"))
export type AddressEvmCanonical = typeof AddressEvmCanonical.Type
export const AddressEvmDisplay = HexChecksum.pipe(S.brand("AddressEvmDisplay"))
export const AddressEvmZkgm = AddressEvmCanonical

// Aptos Address Types
export const AddressAptosCanonical = AddressCanonicalBytes.pipe(
  S.brand("AddressAptosCanonical")
)
export const AddressAptosDisplay = AddressAptosCanonical
export const AddressAptosZkgm = AddressAptosCanonical
