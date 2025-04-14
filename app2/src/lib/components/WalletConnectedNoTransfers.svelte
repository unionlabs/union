<script lang="ts">
  import {cn} from "$lib/utils"
  import type {HTMLAttributes} from "svelte/elements"
  import Button from "./ui/Button.svelte"
  import {goto} from "$app/navigation"
  import OutlineControlPointDuplicate from "./icons/OutlineControlPointDuplicate.svelte"

  type Props = HTMLAttributes<HTMLDivElement> & {
  class?: string
  title?: string
  description?: string
}

const {
  class: className = "",
  title = "No Transfers Found",
  description = "You haven't made any transfers so far.",
  ...rest
}: Props = $props()

const containerClasses = cn("h-[600px] flex justify-center items-center", className)
</script>

<div class={containerClasses} {...rest}>
  <div class="flex flex-col items-center">
    <h2 class="text-xl font-bold">{title}</h2>
    <p class="pb-4">{description}</p>
    <Button variant="secondary" onclick={() => goto("/transfer")}>
      <OutlineControlPointDuplicate />
      Transfer
    </Button>
  </div>
</div>
