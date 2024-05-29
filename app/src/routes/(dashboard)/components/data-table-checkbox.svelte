<script lang="ts">
import type { Writable } from "svelte/store"
import type { HTMLButtonAttributes } from "svelte/elements"
import { Checkbox } from "$lib/components/ui/checkbox/index.ts"

type $$Props = HTMLButtonAttributes & {
  checked: Writable<boolean>
}
export let checked: Writable<boolean>
</script>

<Checkbox bind:checked={$checked} {...$$restProps} />
