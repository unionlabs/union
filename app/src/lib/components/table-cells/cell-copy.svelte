<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import CopyIcon from "virtual:icons/lucide/copy"
import { Button } from "$lib/components/ui/button"
import CheckIcon from "virtual:icons/lucide/check"
import { truncate } from "$lib/utilities/format.ts"
import { copyTextAction } from "$lib/actions/copy.ts"

export let value: string
export let label: string | null = null
export let trunc = 0

let copyClicked = false
const toggleCopy = () => (copyClicked = !copyClicked)
const onCopyClick = () => [toggleCopy(), setTimeout(() => toggleCopy(), 1_500)]

export let buttonClass = ""
export { buttonClass as class }
</script>

<div class="text-start flex items-center gap-2 group cursor-default">
  {#if label}<span class="text-muted-foreground">{label}</span>{/if}
  {#if trunc}
    {truncate(value, trunc)}
  {:else}
    {value}
  {/if}
  <Button
    builders={[{ action: (node) => copyTextAction(node, { text: value }) }]}
    class={cn(
      "bg-transparent hover:bg-transparent opacity-0 group-hover:opacity-100 transition h-5 w-5",
      buttonClass
    )}
    on:click={onCopyClick}
    size="icon"
    variant="ghost"
  >
    {#if copyClicked}
      <CheckIcon class="size-4 text-emerald-500" />
    {:else}
      <CopyIcon class="size-4" />
    {/if}
  </Button>
</div>
