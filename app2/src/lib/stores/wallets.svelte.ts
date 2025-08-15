import { Ucs05 } from "@unionlabs/sdk"
import type { Chain } from "@unionlabs/sdk/schema"
import { Array as A, Match, Option, pipe } from "effect"
import * as S from "effect/Schema"

class WalletsStore {
  evmAddress: Option.Option<Ucs05.EvmDisplay> = $state(Option.none())
  cosmosAddress: Option.Option<Ucs05.CosmosDisplay> = $state(Option.none())
  aptosAddress: Option.Option<Ucs05.AnyDisplay> = $state(Option.none())
  inputAddress: Option.Option<Ucs05.AnyDisplay> = $state(Option.none())

  hasAnyWallet() {
    return (
      Option.isSome(this.evmAddress)
      || Option.isSome(this.cosmosAddress)
      || Option.isSome(this.aptosAddress)
      || Option.isSome(this.inputAddress)
    )
  }

  addInputAddress(address: Ucs05.AnyDisplay | string) {
    this.inputAddress = S.decodeOption(
      S.Union(Ucs05.AnyDisplay, Ucs05.AnyDisplayFromString),
    )(address)
  }

  clearInputAddress() {
    this.inputAddress = Option.none()
  }

  getCanonicalByteAddressList(): ReadonlyArray<Ucs05.CanonicalBytes> {
    return pipe(
      A.getSomes([
        this.evmAddress,
        this.cosmosAddress,
        this.aptosAddress,
      ]),
      A.map(Ucs05.anyDisplayToCanonical),
    )
  }

  getAddressForChain(chain: Chain): Option.Option<Ucs05.AnyDisplay> {
    return Match.value(chain.rpc_type).pipe(
      Match.when("evm", () => this.evmAddress),
      Match.when("cosmos", () =>
        pipe(
          this.cosmosAddress,
          Option.map(Ucs05.anyDisplayToCanonical),
          Option.flatMap(
            S.decodeOption(Ucs05.Bech32FromCanonicalBytesWithPrefix(chain.addr_prefix)),
          ),
          Option.map((address) => Ucs05.CosmosDisplay.make({ address })),
        )),
      Match.when("aptos", () => this.aptosAddress),
      Match.exhaustive,
    )
  }
}

export const wallets = new WalletsStore()
