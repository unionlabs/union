<script lang="ts">
import { toIsoString } from "$lib/utilities/date"
import { Duration } from "svelte-ux"
import { DurationUnits } from "svelte-ux"

interface Props {
  value: Date | undefined
}

let { value }: Props = $props()
</script>


<div>{#if value && value !== undefined}{toIsoString(new Date(value)).split('T')[1].slice(0, 5)}{:else}in transit{/if}</div>
