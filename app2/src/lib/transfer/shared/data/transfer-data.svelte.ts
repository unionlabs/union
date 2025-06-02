import {
  getDerivedReceiverSafe,
  getParsedAmountSafe,
} from "$lib/services/shared";
import { getChannelInfoSafe } from "$lib/services/transfer-ucs03-evm/channel.ts";
import { chains } from "$lib/stores/chains.svelte.ts";
import { channels } from "$lib/stores/channels.svelte.ts";
import { sortedBalancesStore } from "$lib/stores/sorted-balances.svelte.ts";
import { tokensStore } from "$lib/stores/tokens.svelte.ts";
import { uiStore } from "$lib/stores/ui.svelte.ts";
import { wallets } from "$lib/stores/wallets.svelte.ts";
import type { Edition } from "$lib/themes";
import { RawTransferDataSvelte } from "$lib/transfer/shared/data/raw-transfer-data.svelte.ts";
import { signingMode } from "$lib/transfer/signingMode.svelte.ts";
import type { Chain, Channel, Token } from "@unionlabs/sdk/schema";
import { Array as A, Effect, Either, Match, Option, pipe } from "effect";
import { type Address, fromHex, type Hex } from "viem";

export class TransferData {
  raw = new RawTransferDataSvelte();

  // Filter chains by edition before finding specific chains
  filteredChains = $derived(
    chains.data.pipe(
      Option.map((allChains) =>
        allChains.filter((chain) =>
          filterByEdition(chain, uiStore.edition, getEnvironment())
        )
      )
    )
  );

  sourceChain = $derived(
    this.filteredChains.pipe(
      Option.flatMap((cs) =>
        Option.fromNullable(
          cs.find((chain) => chain.chain_id === this.raw.source)
        )
      )
    )
  );

  destinationChain = $derived(
    this.filteredChains.pipe(
      Option.flatMap((cs) =>
        Option.fromNullable(
          cs.find((chain) => chain.chain_id === this.raw.destination)
        )
      )
    )
  );

  baseTokens = $derived(
    this.sourceChain.pipe(
      Option.flatMap((sc) => tokensStore.getData(sc.universal_chain_id))
    )
  );

  quoteTokens = $derived(
    this.destinationChain.pipe(
      Option.flatMap((dc) => tokensStore.getData(dc.universal_chain_id))
    )
  );

  sortedBalances = $derived(
    this.sourceChain.pipe(
      Option.flatMap((sc) =>
        Option.fromNullable(
          Option.isSome(sortedBalancesStore.sortedBalances)
            ? sortedBalancesStore.sortedBalances.value.find(
                (v) => v.chain.universal_chain_id === sc.universal_chain_id
              )
            : undefined
        ).pipe(Option.flatMap((c) => c.tokens))
      )
    )
  );

  baseToken = $derived(
    this.baseTokens.pipe(
      Option.flatMap((tokens) =>
        Option.fromNullable(
          tokens.find((t: Token) => t.denom === this.raw.asset)
        )
      )
    )
  );

  quoteToken = $derived(
    Option.all([
      this.baseToken,
      this.sourceChain,
      this.destinationChain,
      this.quoteTokens,
    ]).pipe(
      Option.flatMap(
        ([baseToken, sourceChain, destinationChain, quoteTokens]) => {
          const baseDenom = baseToken.denom.toLowerCase();

          const maybeUnwrapped = baseToken.wrapping.find(
            (w) =>
              w.wrapped_chain.universal_chain_id ===
                sourceChain.universal_chain_id &&
              w.unwrapped_chain.universal_chain_id ===
                destinationChain.universal_chain_id
          );

          return pipe(
            Option.fromNullable(maybeUnwrapped),
            Option.match({
              onSome: (unwrapped) => Option.some(unwrapped.unwrapped_denom),
              onNone: () =>
                Option.fromNullable(
                  quoteTokens.find((t) =>
                    t.wrapping.some(
                      (w) =>
                        w.unwrapped_denom.toLowerCase() === baseDenom &&
                        w.unwrapped_chain.universal_chain_id ===
                          sourceChain.universal_chain_id &&
                        w.wrapped_chain.universal_chain_id ===
                          destinationChain.universal_chain_id
                    )
                  )?.denom
                ),
            })
          );
        }
      )
    )
  );

  channel = $derived<Option.Option<Channel>>(
    Option.all([channels.data, this.sourceChain, this.destinationChain]).pipe(
      Option.flatMap(([channelsData, sourceChain, destinationChain]) =>
        Match.value({ channelsData, sourceChain, destinationChain }).pipe(
          Match.orElse(() =>
            Option.fromNullable(
              getChannelInfoSafe(
                sourceChain.universal_chain_id,
                destinationChain.universal_chain_id,
                channelsData
              )
            )
          )
        )
      )
    )
  );

  baseTokenBalance = $derived(
    Option.all([this.baseToken, this.sortedBalances]).pipe(
      Option.flatMap(([token, sortedTokens]) =>
        Option.fromNullable(
          sortedTokens.find((t) => t.token.denom === token.denom)
        )
      )
    )
  );

  parsedAmount = $derived(
    this.baseToken.pipe(
      Option.flatMap((bt) => getParsedAmountSafe(this.raw.amount, bt))
    )
  );

  derivedReceiver = $derived(getDerivedReceiverSafe(this.raw.receiver));

  derivedSender = $derived.by(() => {
    if (Option.isNone(this.sourceChain)) {
      return Option.none();
    }

    const sourceChain = this.sourceChain.value;

    if (Option.isSome(wallets.inputAddress) && signingMode.mode === "multi") {
      return wallets.inputAddress;
    } else if (signingMode.mode === "single") {
      return wallets.getAddressForChain(sourceChain);
    }

    return Option.none();
  });

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
            (x) => x.source_universal_chain_id === source.universal_chain_id
          ),
          A.map((x) => x.destination_universal_chain_id),
          A.dedupe,
          (xs) =>
            pipe(
              chains,
              A.filter((chain) => A.contains(xs, chain.universal_chain_id))
            )
        )
      )
    )
  );

  ucs03address = $derived.by<Option.Option<Address>>(() => {
    return Option.all([
      this.sourceChain,
      this.channel,
      Option.fromNullable(
        this.channel
          .pipe(Option.map((c) => c.source_port_id))
          .pipe(Option.getOrUndefined)
      ),
    ]).pipe(
      Option.map(([sourceChain, channel]) => {
        return sourceChain.rpc_type === "cosmos"
          ? (fromHex(
              <`0x${string}`>`${channel.source_port_id}`,
              "string"
            ) as Hex)
          : (channel.source_port_id as Hex);
      })
    );
  });

  flipTransfer = () => {
    if (Option.isSome(this.quoteToken)) {
      this.raw.flip(this.quoteToken.value);
    }
  };
}

const getEnvironment = (): "production" | "staging" | "development" => {
  return pipe(
    Match.value(globalThis?.window?.location?.hostname ?? "localhost").pipe(
      Match.when(
        (hostname) =>
          hostname === "btc.union.build" || hostname === "app.union.build",
        () => "production" as const
      ),
      Match.when(
        (hostname) =>
          hostname === "staging.btc.union.build" ||
          hostname === "staging.app.union.build",
        () => "staging" as const
      ),
      Match.orElse(() => "development" as const)
    )
  );
};

function filterByEdition(
  chain: Chain,
  editionName: Edition,
  environment: string
): boolean {
  return pipe(
    Option.fromNullable(chain.editions),
    Option.match({
      onNone: () => false,
      onSome: (editions) =>
        editions.some((edition: { name: string; environment: string }) => {
          if (edition.name !== editionName) {
            return false;
          }

          return Match.value(edition.environment).pipe(
            Match.when("development", () => environment === "development"),
            Match.when(
              "staging",
              () => environment === "development" || environment === "staging"
            ),
            Match.when("production", () => true),
            Match.orElse(() => false)
          );
        }),
    })
  );
}

export const transferData = new TransferData();
