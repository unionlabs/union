<script lang="ts">
import { Duration } from "svelte-ux"
import { DurationUnits } from "svelte-ux"

export let value: Date | undefined
</script>


<div class="text-muted-foreground text-xs">{#if value && value !== undefined}<Duration totalUnits={3} variant="short" minUnits={DurationUnits.Second} start={value}/>{:else}in transit{/if}</div>
