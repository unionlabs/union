import * as S from "effect/Schema"
import { Hex, HexChecksum, HexFromString } from "./hex.js"
import { Bech32, Bech32FromAddressCanonicalBytesWithPrefix } from "./bech32.js"
import { pipe } from "effect"
import { isAddress } from "viem"

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

// export const AddressCosmosDisplayFromCanonical = flow(
//   Bech32FromAddressCanonicalBytesWithPrefix,
//   S.compose(AddressCosmosDisplay)
// )

export const AddressCosmosZkgmFromAddressCanonicalBytesWithPrefix = (prefix: string) =>
  pipe(
    Bech32FromAddressCanonicalBytesWithPrefix(prefix),
    S.compose(HexFromString),
    S.compose(AddressCosmosZkgm)
  )

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

export const ERC55 = S.NonEmptyString.pipe(
  S.filter(a => isAddress(a, { strict: true }), {
    description: "a string matching ERC-55 in checksum format"
  })
)
export type ERC55 = typeof ERC55.Type

// TODO: rename me
export const ValidAddress = S.Union(ERC55, Bech32)
export type ValidAddress = typeof ValidAddress.Type
