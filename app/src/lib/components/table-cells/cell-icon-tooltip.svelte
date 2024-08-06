<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import EyeIcon from "virtual:icons/lucide/eye"
import CopyIcon from "virtual:icons/lucide/copy"
import CheckIcon from "virtual:icons/lucide/check"
import { copyTextAction } from "$lib/actions/copy"
import { truncate } from "$lib/utilities/format.ts"
import { Button } from "$lib/components/ui/button/index.ts"
import * as Tooltip from "$lib/components/ui/tooltip/index.ts"

export let records: Array<{
  index: number
  hash: string
  label: string
  truncateSize: number
}>

let copyClicked = false
const toggleCopy = () => (copyClicked = !copyClicked)
const onCopyClick = () => [toggleCopy(), setTimeout(() => toggleCopy(), 3_000)]
</script>

<div class="w-full flex items-center group cursor-default mx-3.5">
  <Tooltip.Root let:ids>
    <Tooltip.Trigger asChild let:builder class="size-full">
      <Button
        size="default"
        variant="outline"
        class="size-9 p-1"
        builders={[builder]}
      >
        <EyeIcon class="size-7" />
      </Button>
    </Tooltip.Trigger>
    <Tooltip.Content
      side="right"
      align="start"
      sideOffset={7}
      alignOffset={0}
      class={cn(
        "w-[425px]",
        "px-1 bg-card shadow-lg",
        `h-[${records.length * 11}px]`
      )}
    >
      {#each records as record, index}
        <Button
          size="default"
          variant="secondary"
          on:click={onCopyClick}
          builders={[
            { action: (node) => copyTextAction(node, { text: record.hash }) }
          ]}
          class={cn(
            "flex-1 justify-end w-full px-2",
            "bg-transparent hover:bg-transparent group-hover:opacity-100 transition h-7"
          )}
        >
          <span class="font-medium text-zinc-400 mr-auto">{record.label}</span>
          <span class="font-mono tabular-nums text-white ml-auto mr-3">
            {#if record.truncateSize}
              {truncate(record.hash, record.truncateSize)}
            {:else}
              {record.hash}
            {/if}
          </span>
          {#if copyClicked}
            <CheckIcon class="size-5 text-green-500" />
          {:else}
            <CopyIcon class="size-5 text-primary" />
          {/if}
        </Button>
      {/each}
    </Tooltip.Content>
  </Tooltip.Root>
</div>
