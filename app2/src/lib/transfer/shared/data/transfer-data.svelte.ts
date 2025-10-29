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
import {
  EU_ERC20,
  EU_LST,
  EU_SOLVER_ON_ETH_METADATA,
  EU_SOLVER_ON_UNION_METADATA,
  U_BANK,
  U_ERC20,
  U_SUI,
  U_SOLVER_ON_ETH_METADATA,
  U_SOLVER_ON_UNION_METADATA,
  U_SOLVER_ON_SUI_METADATA
} from "@unionlabs/sdk/Constants"
import * as US from "@unionlabs/sdk/schema"
import { Array as A, Brand, Effect, Match, Option, pipe, String as Str, Struct } from "effect"
import * as B from "effect/Boolean"
import { constant, constFalse } from "effect/Function"
import * as S from "effect/Schema"
import { type Address, fromHex, type Hex, toHex } from "viem"

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

  isSolve = $derived(pipe(
    this.baseToken,
    Option.map((baseToken) =>
      pipe(
        [
          "0x6175", // au
          "0xba5ed44733953d79717f6269357c77718c8ba5ed", // U
          EU_ERC20.address.toLowerCase(), //
          toHex(EU_LST.address),
          // TODO: add eU base
          // TODO: add eU quote
        ],
        A.some((x) => x === baseToken.denom.toLowerCase()),
      )
    ),
    Option.getOrElse(constFalse),
  ))

  quoteToken = $derived.by(() => {
    return Option.all([
      this.baseToken,
      this.sourceChain,
      this.destinationChain,
      this.quoteTokens,
    ]).pipe(
      Option.flatMap(
        ([baseToken, sourceChain, destinationChain, quoteTokens]) => {
          if (this.isSolve) {
            return Match.value([
              Brand.unbranded(baseToken.denom).toLowerCase(),
              destinationChain.rpc_type,
              destinationChain.universal_chain_id,
            ]).pipe(
              Match.when(
                ["0x6175", "evm", Match.any],
                () => U_ERC20,
              ),
              Match.when(
                ["0x6175", "sui", Str.startsWith("sui.")],
                () => U_SUI,
              ),
              Match.when(
                [U_ERC20.address.toLowerCase(), "evm", Match.any],
                () => U_ERC20,
              ),
              Match.when(
                [U_ERC20.address.toLowerCase(), "cosmos", Str.startsWith("union.")],
                () => U_BANK,
              ),
              Match.when(
                [EU_ERC20.address.toLowerCase(), "evm", Match.any],
                () => EU_ERC20,
              ),
              Match.when(
                [EU_ERC20.address.toLowerCase(), "cosmos", Match.any],
                () => Token.Cw20.make({ address: EU_LST.address }),
              ),
              Match.when(
                [toHex(EU_LST.address.toLowerCase()), "evm", Match.any],
                () => Token.Erc20.make({ address: EU_ERC20.address }),
              ),
              Match.when(
                [toHex(EU_LST.address.toLowerCase()), "cosmos", Match.any],
                () => Token.Cw20.make({ address: EU_LST.address }),
              ),
              Match.option,
            )
          }

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
        // Override kind to "solve" for Union
        if (this.isSolve) {
          return Option.some("solve" as const)
        }

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

  metadata = $derived(
    Option.all([this.kind, this.baseToken, this.destinationChain]).pipe(
      Option.flatMap(([kind, baseToken, destChain]) =>
        Match.value([kind, baseToken.denom, destChain.rpc_type, destChain.universal_chain_id]).pipe(
          Match.whenOr(
            ["solve", "0x6175", "evm", Match.any],
            ["solve", U_ERC20.address.toLowerCase(), "evm", Match.any],
            () => Option.some(U_SOLVER_ON_ETH_METADATA),
          ),
          Match.whenOr(
            ["solve", "0x6175", "sui", Str.startsWith("sui.")],
            () => Option.some(U_SOLVER_ON_SUI_METADATA),
          ),
          Match.when(
            ["solve", U_ERC20.address.toLowerCase(), "cosmos", Str.startsWith("union.")],
            () => Option.some(U_SOLVER_ON_UNION_METADATA),
          ),
          Match.when(
            ["solve", EU_ERC20.address.toLowerCase(), "evm", Str.startsWith("union.")],
            () => Option.some(EU_SOLVER_ON_ETH_METADATA),
          ),
          Match.when(
            ["solve", EU_ERC20.address.toLowerCase(), "cosmos", Str.startsWith("union.")],
            () => Option.some(EU_SOLVER_ON_UNION_METADATA),
          ),
          Match.when(
            ["solve", toHex(EU_LST.address.toLowerCase()), "evm", Str.startsWith("ethereum.")],
            () => Option.some(EU_SOLVER_ON_ETH_METADATA),
          ),
          Match.when(
            ["solve", U_ERC20.address.toLowerCase(), "sui", Str.startsWith("sui.")],
            () => Option.some(U_SOLVER_ON_SUI_METADATA),
          ),
          Match.when(
            ["solve", Match.any, Match.any, Match.any],
            () => Option.none(),
          ),

          // Match.when(
          //   ["solve", "eU (tohex)", Match.any],
          //   () => EU_FROM_UNION_SOLVER_METADATA,
          // ),
          // Match.when(
          //   ["solve", EU_ERC20.address.toLowerCase(), Match.any],
          //   () => EU_FROM_UNION_SOLVER_METADATA,
          // ),
          Match.orElse(() => Option.some(undefined)),
        )
      ),
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
    Option.map(A.contains<"canonical" | "tokenorder-v2">("tokenorder-v2")),
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
    // if (Option.isSome(this.quoteToken)) {
    //   this.raw.flip(this.quoteToken.value)
    // }
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
