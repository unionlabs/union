import * as S from "effect/Schema"
import { Hex, HexChecksum } from "./hex.js"
import { Bech32, Bech32FromHexWithPrefix } from "./bech32.js"

// For Reference, see: https://docs.union.build/ucs/05
// We always store bytes arrays as hex-encoded strings
export const AddressCanonicalBytes = Hex.pipe(S.brand("CanonicalBytes"))
export type AddressCanonicalBytes = typeof AddressCanonicalBytes.Type

// Cosmos Address Types
export const AddressCosmosCanonical = AddressCanonicalBytes.pipe(S.brand("AddressCosmosCanonical"))
export type AddressCosmosCanonical = typeof AddressCosmosCanonical.Type

export const AddressCosmosDisplay = Bech32.pipe(S.brand("AddressCosmosDisplay"))
export type AddressCosmosDisplay = typeof AddressCosmosDisplay.Type

export const AddressCosmosZkgm = Hex.pipe(S.brand("AddressCosmosZkgm")) // TODO: Hex<Bech32<Hrp, Cosmos.Canonical>>
export type AddressCosmosZkgm = typeof AddressCosmosZkgm.Type

/**
 * TODO: how to get appropriate prefix (?)
 */
export const AddressCosmosDisplayFromHex = S.compose(
  Bech32FromHexWithPrefix("cosmos"),
  AddressCosmosDisplay,
)
export type AddressCosmosDisplayFromHex = typeof AddressCosmosDisplayFromHex.Type

// Evm Address Types
export const AddressEvmCanonical = AddressCanonicalBytes.pipe(S.brand("AddressEvmCanonical"))
export type AddressEvmCanonical = typeof AddressEvmCanonical.Type

export const AddressEvmDisplay = HexChecksum.pipe(S.brand("AddressEvmDisplay"))
export type AddressEvmDisplay = typeof AddressEvmDisplay.Type

export const AddressEvmZkgm = AddressEvmCanonical.pipe(S.brand("AddressEvmZkgm"))
export type AddressEvmZkgm = typeof AddressEvmZkgm.Type

// Aptos Address Types
export const AddressAptosCanonical = AddressCanonicalBytes.pipe(S.brand("AddressAptosCanonical"))
export const AddressAptosDisplay = AddressAptosCanonical
export const AddressAptosZkgm = AddressAptosCanonical
