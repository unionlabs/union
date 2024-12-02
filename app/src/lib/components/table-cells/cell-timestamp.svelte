<script lang="ts">
import { toPrettyDateTimeFormat } from "$lib/utilities/date.ts"

export let value: string | undefined
</script>

{#if value}{toPrettyDateTimeFormat(value, { local: true })}{:else}-{/if}
