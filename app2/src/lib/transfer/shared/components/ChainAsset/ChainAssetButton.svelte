<script lang="ts">
import LoadingSpinnerIcon from "$lib/components/icons/LoadingSpinnerIcon.svelte"
import SharpChevronDownIcon from "$lib/components/icons/SharpChevronDownIcon.svelte"
import AddressComponent from "$lib/components/model/AddressComponent.svelte"
import Label from "$lib/components/ui/Label.svelte"
import { chainLogoMap } from "$lib/constants/chain-logos"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte"
import { cn } from "$lib/utils/index.js"
import type { Chain } from "@unionlabs/sdk/schema"
import { Array as A, Match, Option, pipe } from "effect"

type Props = {
  type: "source" | "destination"
  onClick: () => void
}

const { type, onClick }: Props = $props()

const selectedChain: Option.Option<Chain> = $derived(
  pipe(
    Match.value(type),
    Match.when("source", () => {
      void transferData.raw.source
      return transferData.sourceChain
    }),
    Match.when("destination", () => {
      void transferData.raw.destination
      return transferData.destinationChain
    }),
    Match.exhaustive,
  ),
)

const isChainLoading: boolean = $derived(
  pipe(
    Match.value(type),
    Match.when(
      "source",
      () => Boolean(transferData.raw.source && Option.isNone(transferData.sourceChain)),
    ),
    Match.when(
      "destination",
      () => Boolean(transferData.raw.destination && Option.isNone(transferData.destinationChain)),
    ),
    Match.exhaustive,
  ),
)
</script>

<div class="w-full">
  <div class="flex items-center justify-between">
    <Label class="pb-1">{type}</Label>
    <Label>
      {#if type === "source" && Option.isSome(transferData.sourceChain)
          && Option.isSome(transferData.derivedSender)}
        <AddressComponent
          truncate
          class="text-accent"
          truncateChars={8}
          address={transferData.derivedSender.value}
          chain={transferData.sourceChain.value}
        />
      {:else if type === "destination" && Option.isSome(transferData.destinationChain)
          && Option.isSome(transferData.derivedReceiver)}
        <AddressComponent
          truncate
          class="text-accent"
          truncateChars={8}
          address={transferData.derivedReceiver.value}
          chain={transferData.destinationChain.value}
        />
      {/if}
    </Label>
  </div>

  <button
    onclick={onClick}
    class={cn(
      "w-full h-14 rounded-md bg-zinc-800/70 text-zinc-200",
      "hover:bg-zinc-800 hover:border-zinc-500",
      "focus:outline-none focus:ring-1 focus:ring-accent",
      "disabled:opacity-50 disabled:cursor-not-allowed",
      "transition-all duration-200 cursor-pointer",
    )}
  >
    <div class="flex items-center">
      {#if isChainLoading}
        <div class="flex gap-2 items-center justify-between p-2 flex-1">
          <div class="w-8 h-8 flex items-center bg-zinc-500 text-white rounded-full justify-center">
            <LoadingSpinnerIcon />
          </div>
          <span class="text-zinc-400">Loading...</span>
          <div class="text-transparent">
            <SharpChevronDownIcon />
          </div>
        </div>
      {:else if Option.isNone(selectedChain)}
        <!-- No Chain Selected -->
        <div class="flex gap-2 items-center justify-between p-3 flex-1">
          <div class="w-8 h-8 flex items-center bg-zinc-700 rounded-full justify-center"></div>
          <span class="text-zinc-400 flex-1 text-start">Select</span>
          <SharpChevronDownIcon class="size-6" />
        </div>
      {:else}
        <!-- Chain Selected -->
        <div class="flex gap-2 items-center justify-between p-3 flex-1 w-full">
          <!--LOGO-->
          {#if selectedChain.value.universal_chain_id}
            {@const chainLogo = chainLogoMap.get(selectedChain.value.universal_chain_id)}
            {@const selectedAsset = pipe(
            Match.value(type),
            Match.when("source", () =>
              pipe(
                transferData.baseToken,
                Option.map(x => x.representations),
                Option.flatMap(A.head),
              )),
            Match.when("destination", () =>
              pipe(
                transferData.baseToken,
                Option.map(x => x.representations),
                Option.flatMap(A.head),
              )),
            Match.exhaustive,
          )}
            {@const validSelectedAsset = Option.isSome(selectedAsset)
            && Option.isSome(selectedAsset.value.logo_uri)}
            {#if chainLogo?.color}
              <div class="flex items-center">
                <div class="relative size-8 flex items-center justify-center overflow-visible mr-2">
                  {#if validSelectedAsset}
                    <img
                      src={selectedAsset.value.logo_uri.value}
                      alt={selectedAsset.value.name}
                      class="h-8 w-8 asset-mask"
                    >
                  {/if}
                  {#if validSelectedAsset}
                    <div class="absolute inline-flex items-center justify-center w-4 h-4 rounded-full bottom-0 -end-2 bg-clip-text bg-white">
                      <img
                        class="h-4 w-4 object-fill"
                        src={chainLogo.color}
                        alt={selectedChain.value.display_name}
                      />
                    </div>
                  {/if}
                </div>
              </div>
            {/if}
          {/if}

          {#if type === "source" && transferData.raw.asset
            && Option.isNone(transferData.baseToken)}
            <!-- Asset Loading (only for source) -->
            <span class="flex items-center">
              <span>Loading...</span>
            </span>
          {:else if Option.isSome(transferData.baseToken)}
            <!-- Selected Asset (both source and destination) -->
            <!-- Show the asset, grayed out for destination type -->
            <div
              class={cn(
                type === "destination" ? "truncate" : "truncate",
                "flex flex-col items-start w-full",
              )}
            >
              <p class="leading-4 font-bold">
                {
                  transferData.baseToken.value.representations[0]?.symbol
                  ?? transferData.baseToken.value.denom
                }
              </p>
              {#if Option.isSome(transferData.sourceChain)}
                <p class="text-xs text-zinc-400">
                  {
                    type === "source"
                    ? transferData.sourceChain.value.display_name
                    : Option.getOrUndefined(transferData.destinationChain)?.display_name
                  }
                </p>
              {/if}
            </div>
          {:else if type === "source"}
            <span class="text-zinc-400 flex-1 text-start">Select</span>
          {:else}
            <span class="text-zinc-400 flex-1 text-start">No asset</span>
          {/if}
          <SharpChevronDownIcon class="size-6" />
        </div>
      {/if}
    </div>
  </button>
</div>

<style>
.asset-mask {
  --diameter: calc(var(--spacing) * 2.5);
  --offset-x: calc(100% - var(--diameter) * 0);
  --offset-y: calc(100% - var(--diameter) * 2.4/3);

  mask-image: radial-gradient(
    circle var(--diameter) at var(--offset-x) var(--offset-y),
    transparent 90%,
    white 100%
  );
  mask-composite: exclude;
  -webkit-mask-composite: destination-out;
}
</style>
