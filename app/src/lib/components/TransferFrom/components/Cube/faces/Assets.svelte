<script lang="ts">
  import type {IntentStore} from "$lib/components/TransferFrom/transfer/intents.ts";
  import type {ValidationStoreAndMethods} from "$lib/components/TransferFrom/transfer/validation.ts";
  import type {Readable} from "svelte/store";
  import type {ContextStore} from "$lib/components/TransferFrom/transfer/context.ts";
  import type {CubeFaces} from "$lib/components/TransferFrom/types.ts";
  import {showUnsupported} from "$lib/stores/user.ts";
  import {getSupportedAsset} from "$lib/utilities/helpers.ts";
  import {truncate} from "$lib/utilities/format.ts";
  import {formatUnits} from "viem";
  import {Button} from "$lib/components/ui/button";

  interface Props {
    stores: {
      intents: IntentStore
      validation: ValidationStoreAndMethods
      context: Readable<ContextStore>
    }
    rotateTo: (face: CubeFaces) => void
  }

  export let stores: Props["stores"]
  export let rotateTo: Props["rotateTo"]

  let {intents, validation, context} = stores

  function setAsset(address: string) {
    intents.updateField('asset', address)
    rotateTo("intentFace")
  }

</script>

<div class="flex flex-col h-full w-full">
  <div class="flex-1 overflow-y-auto">
    {#each $context.balances as asset}
      {@const supportedAsset = getSupportedAsset($context.sourceChain, asset.address)}
      {#if $showUnsupported || supportedAsset}
        <div class="pb-2 flex flex-col justify-start">
          <Button
                  variant="ghost"
                  class="px-4 py-2 w-full rounded-none flex justify-between items-center"
                  on:click={() => setAsset(asset.address)}
          >
            <div class:opacity-30={!supportedAsset}>
              {truncate((supportedAsset?.display_symbol || asset?.symbol || ''), 6) || 'Unknown symbol'}
            </div>
            <p class="text-lg font-black" class:opacity-30={!supportedAsset}>
              {formatUnits(asset.balance, supportedAsset?.decimals ?? 0)}
            </p>
          </Button>
        </div>
      {/if}
    {/each}
  </div>

  <div class="mt-4">
    <Button on:click={() => rotateTo("intentFace")} class="w-full">
      Back
    </Button>
  </div>
</div>
