<script lang="ts">
import { cn } from "$lib/utils"
import { BigDecimal, Option as O, pipe } from "effect"

interface Props {
  value: number
  onchange: (value: number) => void
  class?: string
}

let {
  value = 1,
  onchange,
  class: className = "",
}: Props = $props()

type SlippageKey = "1" | "2" | "custom"

const presetOptions: Array<{ key: SlippageKey; label: string; value?: number }> = [
  { key: "1", label: "1", value: 1 },
  { key: "2", label: "2", value: 2 },
  { key: "custom", label: "Custom" },
]

const getKeyFromValue = (incoming: number): SlippageKey =>
  incoming === 1 || incoming === 2 ? String(incoming) as SlippageKey : "custom"

let selectedKey = $state<SlippageKey>(getKeyFromValue(value))
let customInput = $state(selectedKey === "custom" ? value.toString() : "")

$effect(() => {
  const nextKey = getKeyFromValue(value)
  selectedKey = nextKey
  customInput = nextKey === "custom" ? value.toString() : ""
})

const handleSegmentClick = (key: SlippageKey) => {
  if (key === "custom") {
    selectedKey = "custom"
    customInput = value.toString()
    return
  }

  const preset = presetOptions.find(option => option.key === key)?.value
  if (preset == null) {
    return
  }

  selectedKey = key
  customInput = ""
  onchange(preset)
}

const parseSlippage = (input: string): O.Option<number> =>
  pipe(
    BigDecimal.fromString(input),
    O.map(BigDecimal.unsafeToNumber),
    O.filter(num => num >= 0 && num <= 100),
  )

const handleCustomChange = (e: Event) => {
  const target = e.target as HTMLInputElement
  const inputValue = target.value

  // Allow empty input while typing
  if (inputValue === "") {
    customInput = ""
    return
  }

  pipe(
    parseSlippage(inputValue),
    O.match({
      onNone: () => {},
      onSome: (parsed) => {
        selectedKey = "custom"
        customInput = inputValue
        onchange(parsed)
      },
    }),
  )
}

const handleCustomBlur = () => {
  // If empty, reset to previous value
  if (!customInput) {
    customInput = value.toString()
    onchange(value)
    return
  }

  pipe(
    parseSlippage(customInput),
    O.match({
      onNone: () => {
        customInput = value.toString()
        onchange(value)
      },
      onSome: (parsed) => {
        if (parsed <= 0 || parsed > 100) {
          customInput = value.toString()
          onchange(value)
        }
      },
    }),
  )
}
</script>

<div class={cn("space-y-1.5", className)}>
  <div class="flex items-center justify-between">
    <span class="text-[11px] text-zinc-500 font-medium">Slippage</span>
    <div class="flex items-center gap-1">
      {#each presetOptions as option}
        {#if option.key === "custom" && selectedKey === "custom"}
          <div class="relative w-[60px]">
            <input
              type="text"
              value={customInput}
              oninput={handleCustomChange}
              onblur={handleCustomBlur}
              inputmode="decimal"
              class="h-7 w-full rounded border border-accent/30 bg-transparent px-2 text-[11px] font-medium text-zinc-100 placeholder:text-zinc-500 focus:border-accent/60 focus:outline-none focus:ring-1 focus:ring-accent/30"
              placeholder="1.5"
            />
            <span
              class="pointer-events-none absolute inset-y-0 right-2 flex items-center text-[10px] text-zinc-500"
            >%</span>
          </div>
        {:else}
          <button
            type="button"
            class={cn(
              "h-7 min-w-[48px] cursor-pointer px-2 text-[11px] font-medium rounded border border-transparent transition-all",
              option.key === selectedKey
                ? "border-accent/30 bg-accent/10 text-accent"
                : "text-zinc-500 hover:text-zinc-300 hover:border-zinc-700",
            )}
            onclick={() => handleSegmentClick(option.key)}
          >
            {option.key === "custom" ? option.label : `${option.label}%`}
          </button>
        {/if}
      {/each}
    </div>
  </div>
</div>
