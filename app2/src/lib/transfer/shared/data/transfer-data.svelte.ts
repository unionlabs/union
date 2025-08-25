import * as AppRuntime from "$lib/runtime"
import { runSync } from "$lib/runtime"
import { getParsedAmountSafe } from "$lib/services/shared"
import { getChannelInfo } from "$lib/services/transfer-ucs03-evm/channel"
import { chains } from "$lib/stores/chains.svelte"
import { channels } from "$lib/stores/channels.svelte"
import { sortedBalancesStore } from "$lib/stores/sorted-balances.svelte"
import { tokensStore } from "$lib/stores/tokens.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import { wallets } from "$lib/stores/wallets.svelte"
import type { Edition } from "$lib/themes"
import { RawTransferDataSvelte } from "$lib/transfer/shared/data/raw-transfer-data.svelte"
import { signingMode } from "$lib/transfer/signingMode.svelte"
import { Token, TokenOrder, Ucs05 } from "@unionlabs/sdk"
import * as US from "@unionlabs/sdk/schema"
import { Array as A, Brand, Effect, Match, Option, pipe, Struct } from "effect"
import * as B from "effect/Boolean"
import { constant } from "effect/Function"
import * as S from "effect/Schema"
import { type Address, fromHex, type Hex } from "viem"

export class TransferData {
  raw = new RawTransferDataSvelte()

  net = $state<"testnet" | "mainnet" | "all">("mainnet")

  // Filter chains by edition before finding specific chains
  filteredChains = $derived(
    chains.data.pipe(
      Option.map((allChains) =>
        allChains.filter((chain) => filterByEdition(chain, uiStore.edition, getEnvironment()))
      ),
    ),
  )

  sourceChain = $derived(
    this.filteredChains.pipe(
      Option.flatMap((cs) =>
        Option.fromNullable(
          cs.find((chain) => chain.chain_id === this.raw.source),
        )
      ),
    ),
  )

  destinationChain = $derived(
    this.filteredChains.pipe(
      Option.flatMap((cs) =>
        Option.fromNullable(
          cs.find((chain) => chain.chain_id === this.raw.destination),
        )
      ),
    ),
  )

  baseTokens = $derived(
    this.sourceChain.pipe(
      Option.flatMap((sc) => tokensStore.getData(sc.universal_chain_id)),
    ),
  )

  quoteTokens: Option.Option<readonly US.Token[]> = $derived.by(() => {
    return this.destinationChain.pipe(
      Option.flatMap((dc) => tokensStore.getData(dc.universal_chain_id)),
    )
  })

  sortedBalances = $derived(
    this.sourceChain.pipe(
      Option.flatMap((sc) =>
        Option.fromNullable(
          Option.isSome(sortedBalancesStore.sortedBalances)
            ? sortedBalancesStore.sortedBalances.value.find(
              (v) => v.chain.universal_chain_id === sc.universal_chain_id,
            )
            : undefined,
        ).pipe(Option.flatMap((c) => c.tokens))
      ),
    ),
  )

  baseToken = $derived(
    this.baseTokens.pipe(
      Option.flatMap((tokens) =>
        Option.fromNullable(
          tokens.find((t: US.Token) => t.denom === this.raw.asset),
        )
      ),
    ),
  )

  quoteToken = $derived.by(() => {
    const baseTokenDenom = Option.getOrUndefined(
      Option.map(this.baseToken, x => Brand.unbranded(x.denom)),
    )
    if (
      baseTokenDenom === "0x6175" || baseTokenDenom === "0xba5ed44733953d79717f6269357c77718c8ba5ed"
    ) {
      return Option.some(
        Token.Erc20.make({ address: "0xba5eD44733953d79717F6269357C77718C8Ba5ed" }),
      )
    }

    return Option.all([
      this.baseToken,
      this.sourceChain,
      this.destinationChain,
      this.quoteTokens,
    ]).pipe(
      Option.flatMap(
        ([baseToken, sourceChain, destinationChain, quoteTokens]) => {
          const baseDenom = baseToken.denom.toLowerCase()

          const maybeUnwrapped = baseToken.wrapping.find(
            (w) =>
              w.wrapped_chain.universal_chain_id
                === sourceChain.universal_chain_id
              && w.unwrapped_chain.universal_chain_id
                === destinationChain.universal_chain_id,
          )

          return pipe(
            Option.fromNullable(maybeUnwrapped),
            Option.match({
              onSome: (unwrapped) => Option.some(unwrapped.unwrapped_denom),
              onNone: () =>
                Option.fromNullable(
                  quoteTokens.find((t) =>
                    t.wrapping.some(
                      (w) =>
                        w.unwrapped_denom.toLowerCase() === baseDenom
                        && w.unwrapped_chain.universal_chain_id
                          === sourceChain.universal_chain_id
                        && w.wrapped_chain.universal_chain_id
                          === destinationChain.universal_chain_id,
                    )
                  )?.denom,
                ),
            }),
            Option.flatMap((raw) =>
              S.decodeOption(Token.AnyFromEncoded(destinationChain.rpc_type))(raw)
            ),
          )
        },
      ),
    )
  })

  channel = $derived<Option.Option<US.Channel>>(
    Option.all([channels.data, this.sourceChain, this.destinationChain]).pipe(
      Option.flatMap(([channelsData, sourceChain, destinationChain]) =>
        runSync(
          getChannelInfo(
            sourceChain.universal_chain_id,
            destinationChain.universal_chain_id,
            channelsData,
          ).pipe(Effect.option),
        )
      ),
    ),
  )

  representations = $derived(
    Option.all([this.baseToken, this.sourceChain, this.destinationChain, this.channel]).pipe(
      Option.map(([baseToken, sourceChain, destinationChain, channel]) => {
        return baseToken.wrapping.filter(wrapping =>
          wrapping.wrapped_chain.universal_chain_id === sourceChain.universal_chain_id
          && wrapping.unwrapped_chain.universal_chain_id === destinationChain.universal_chain_id
          && wrapping.destination_channel_id === channel.source_channel_id
        )
      }),
    ),
  )

  kind = $derived<Option.Option<TokenOrder.Kind>>(
    Option.all([this.baseToken, this.sourceChain, this.destinationChain]).pipe(
      Option.flatMap(([baseToken, sourceChain, destinationChain]) => {
        const sourceId = sourceChain.universal_chain_id
        const destId = destinationChain.universal_chain_id

        return pipe(
          baseToken.wrapping,
          A.findFirst(wrapping =>
            wrapping.wrapped_chain.universal_chain_id === sourceId
            && wrapping.unwrapped_chain.universal_chain_id === destId
          ),
          Option.map(() => "unescrow" as const),
          Option.orElseSome(() => "escrow" as const),
        )
      }),
    ),
  )

  destChannel = $derived<Option.Option<US.Channel>>(
    Option.all([channels.data, this.sourceChain, this.destinationChain]).pipe(
      Option.flatMap(([channelsData, sourceChain, destinationChain]) =>
        runSync(
          getChannelInfo(
            destinationChain.universal_chain_id,
            sourceChain.universal_chain_id,
            channelsData,
          ).pipe(Effect.option),
        )
      ),
    ),
  )

  version = $derived(pipe(
    this.channel,
    Option.tap((x) => {
      return Option.some(x)
    }),
    Option.map(Struct.get("tags")),
    Option.map(A.contains("tokenorder-v2")),
    Option.map(B.match({
      onTrue: constant(2 as const),
      onFalse: constant(1 as const),
    })),
    Option.tap((x) => {
      return Option.some(x)
    }),
  ))

  baseTokenBalance = $derived(
    Option.all([this.baseToken, this.sortedBalances]).pipe(
      Option.flatMap(([token, sortedTokens]) =>
        Option.fromNullable(
          sortedTokens.find((t) => t.token.denom === token.denom),
        )
      ),
    ),
  )

  parsedAmount = $derived(
    this.baseToken.pipe(
      Option.flatMap((bt) => getParsedAmountSafe(this.raw.amount, bt)),
    ),
  )

  derivedReceiver: Option.Option<Ucs05.AnyDisplay> = $derived.by(() => {
    return AppRuntime.runSync(pipe(
      S.decode(S.Union(Ucs05.AnyDisplay, Ucs05.AnyDisplayFromString))(this.raw.receiver),
      Effect.option,
    ))
  })

  derivedSender: Option.Option<Ucs05.AnyDisplay> = $derived.by(() => {
    if (Option.isNone(this.sourceChain)) {
      return Option.none()
    }

    const sourceChain = this.sourceChain.value

    if (Option.isSome(wallets.inputAddress) && signingMode.mode === "multi") {
      return wallets.inputAddress
    } else if (signingMode.mode === "single") {
      return wallets.getAddressForChain(sourceChain)
    }

    return Option.none()
  })

  /**
   * Based on source or destination fulfilled, return channels open or closed.
   * Now uses filtered chains instead of all chains.
   */
  destinationChains = $derived(
    pipe(
      Option.all({
        channels: channels.data,
        chains: this.filteredChains,
        source: this.sourceChain,
      }),
      Option.map(({ channels, chains, source }) =>
        pipe(
          channels,
          A.filter(
            (x) => x.source_universal_chain_id === source.universal_chain_id,
          ),
          A.map((x) => x.destination_universal_chain_id),
          A.dedupe,
          (xs) =>
            pipe(
              chains,
              A.filter((chain) => A.contains(xs, chain.universal_chain_id)),
            ),
        )
      ),
    ),
  )

  ucs03address = $derived.by<Option.Option<Address>>(() => {
    return Option.all([
      this.sourceChain,
      this.channel,
      Option.fromNullable(
        this.channel
          .pipe(Option.map((c) => c.source_port_id))
          .pipe(Option.getOrUndefined),
      ),
    ]).pipe(
      Option.map(([sourceChain, channel]) => {
        return sourceChain.rpc_type === "cosmos"
          ? (fromHex(
            <`0x${string}`> `${channel.source_port_id}`,
            "string",
          ) as Hex)
          : (channel.source_port_id as Hex)
      }),
    )
  })

  flipTransfer = () => {
    if (Option.isSome(this.quoteToken)) {
      this.raw.flip(this.quoteToken.value)
    }
  }
}

const getEnvironment = (): "production" | "staging" | "development" =>
  Match.value(globalThis?.window?.location?.hostname ?? "localhost").pipe(
    Match.when(
      (hostname) => hostname === "btc.union.build" || hostname === "app.union.build",
      () => "production" as const,
    ),
    Match.when(
      (hostname) =>
        hostname === "staging.btc.union.build"
        || hostname === "staging.app.union.build",
      () => "staging" as const,
    ),
    Match.orElse(() => "development" as const),
  )

function filterByEdition(
  chain: US.Chain,
  editionName: Edition,
  environment: string,
): boolean {
  if (chain.chain_id === "union-testnet-10") {
    return true
  } // XXX: remove me

  return pipe(
    Option.fromNullable(chain.editions),
    Option.match({
      onNone: () => false,
      onSome: (editions) =>
        editions.some((edition: { name: string; environment: string }) => {
          if (edition.name !== editionName && (editionName !== "app")) {
            return false
          }

          return Match.value(edition.environment).pipe(
            Match.when("development", () => environment === "development"),
            Match.when(
              "staging",
              () => environment === "development" || environment === "staging",
            ),
            Match.when("production", () => true),
            Match.orElse(() => false),
          )
        }),
    }),
  )
}

export const transferData = new TransferData()
