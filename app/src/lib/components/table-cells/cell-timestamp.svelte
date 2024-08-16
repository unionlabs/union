<script lang="ts">
import { toIsoString } from "$lib/utilities/date"
import { Duration } from "svelte-ux"
import { DurationUnits } from "svelte-ux"
import { toPrettyDateTimeFormat } from "$lib/utilities/date.ts"

export let value: Date | undefined
</script>

{#if value}{toPrettyDateTimeFormat(value, { local: true })}{:else}-{/if}
