<script lang="ts">
import { Duration } from "svelte-ux"

let _class: string
export let start: Date
export let minUnits: number
export let totalUnits: number
export let variant: "long" | "short" = "short"
export { _class as class }
</script>

<div class='min-w-18'>
  <Duration {totalUnits} {variant} {minUnits} {start} class={_class} {...$$restProps} />
</div>

<style lang="postcss"></style>
