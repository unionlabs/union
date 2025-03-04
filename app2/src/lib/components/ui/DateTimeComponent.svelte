<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import { cn } from "$lib/utils"
import { DateTime, Effect } from "effect"

type Props = HTMLAttributes<HTMLTimeElement> & {
  value: DateTime.DateTime
  class?: string
  showSeconds?: boolean
}

const { value, class: className = "", showSeconds = true, ...rest }: Props = $props()

const classes = cn("text-zinc-400", className)

const timeFormat = new Intl.DateTimeFormat("en-US", {
  hour: "2-digit",
  minute: "2-digit",
  second: showSeconds ? "2-digit" : undefined,
  hour12: false // Set to false for 24-hour format
})

const dateFormat = new Intl.DateTimeFormat("en-CA", {
  year: "numeric",
  month: "2-digit",
  day: "2-digit"
})

const formatDate = (value: DateTime.DateTime) =>
  Effect.gen(function* () {
    const zonedNow = yield* DateTime.nowInCurrentZone
    const today = DateTime.startOf(zonedNow, "day")
    const yesterday = DateTime.subtract(today, { days: 1 })
    const zonedValue = DateTime.setZone(value, DateTime.zoneMakeLocal())
    const compareDate = DateTime.startOf(zonedValue, "day")

    if (DateTime.Equivalence(compareDate, today)) {
      return `Today ${DateTime.formatIntl(zonedValue, timeFormat)}`
    }
    if (DateTime.Equivalence(compareDate, yesterday)) {
      return `Yesterday ${DateTime.formatIntl(zonedValue, timeFormat)}`
    }

    return `${DateTime.formatIntl(zonedValue, dateFormat)} ${DateTime.formatIntl(zonedValue, timeFormat)}`
  }).pipe(DateTime.withCurrentZoneLocal)

const formattedDate = $derived(Effect.runSyncExit(formatDate(value)))

// Keep ISO format for the datetime attribute for accessibility
const isoDate = $derived(DateTime.formatIso(value))
</script>

{#if formattedDate._tag === "Success"}
  <time datetime={isoDate} class={classes} {...rest}>
    {formattedDate.value}
  </time>
{:else}
  <time datetime={isoDate} class={classes} {...rest}>
    {isoDate}
  </time>
{/if}
