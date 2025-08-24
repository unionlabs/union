<script lang="ts">
import Truncate from "$lib/components/ui/Truncate.svelte"
import { mapOption } from "$lib/utils/snippets.svelte"
import { TokenWrapping } from "@unionlabs/sdk/schema"
import { Option } from "effect"
import { transferData } from "../../data/transfer-data.svelte"

type Props = {
  onSelect: (wrapping: TokenWrapping) => void
}

let props: Props = $props()

const hasMultipleReps = transferData.representations.pipe(
  Option.map(reps => reps.length > 1),
  Option.getOrElse(() => false),
)

const onSelect = (wrapping: TokenWrapping) => {
  transferData.raw.updateField("quoteToken", wrapping.unwrapped_denom)
  props.onSelect(wrapping)
}
</script>

{#snippet renderWrappings(wrappings: TokenWrapping[])}
  {#each wrappings as wrapping}
    <button
      class="flex px-5 py-2 items-center text-white transition-all duration-100 cursor-pointer hover:bg-zinc-800 rounded"
      onclick={() => onSelect(wrapping)}
      disabled={!Option.isSome(transferData.sourceChain)}
    >
      <div>
        <Truncate
          value={wrapping.unwrapped_denom}
          maxLength={18}
          showCopy={false}
        />
      </div>
    </button>
  {/each}
{/snippet}

<div>
  <div class="p-5 text-zinc-400">
    The token you are sending has multiple representations on the destination. Please select the
    contract address of the asset you want to receive.
  </div>
  <div class="flex flex-col">
    {@render mapOption(transferData.representations, renderWrappings)}
  </div>
</div>
