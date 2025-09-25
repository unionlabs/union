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

let isCustom = $state(![1, 2].includes(value))
let customInput = $state("")
let showCustomInput = $state(false)

// Initialize custom values if needed
$effect(() => {
  if (![1, 2].includes(value)) {
    isCustom = true
    customInput = value.toString()
    showCustomInput = false // Keep input hidden initially for custom values
  }
})

const presetOptions = [
  { label: "1", value: 1 },
  { label: "2", value: 2 },
]

const handlePresetClick = (presetValue: number) => {
  isCustom = false
  showCustomInput = false
  customInput = ""
  onchange(presetValue)
}

const handleCustomClick = () => {
  isCustom = true
  showCustomInput = true
  if (!customInput) {
    customInput = value.toString()
  }
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
  <div class="flex justify-between items-center">
    <span class="text-[11px] text-zinc-500 font-medium">Slippage</span>
    <div class="flex items-center gap-0.5">
      {#each presetOptions as option}
        <button
          class={cn(
            "px-1.5 py-0.5 text-[11px] font-medium rounded transition-all",
            !isCustom && value === option.value
              ? "bg-accent/20 text-accent"
              : "text-zinc-500 hover:text-zinc-400",
          )}
          onclick={() => handlePresetClick(option.value)}
        >
          {option.label}%
        </button>
      {/each}

      <div class="relative">
        {#if showCustomInput}
          <div class="flex items-center">
            <input
              type="text"
              value={customInput}
              oninput={handleCustomChange}
              onblur={handleCustomBlur}
              class={cn(
                "w-12 px-1 py-0.5 text-[11px] font-medium rounded transition-all",
                "bg-zinc-800 text-zinc-100 border border-zinc-700",
                "focus:border-accent/40 focus:outline-none focus:ring-1 focus:ring-accent/20",
              )}
              placeholder="1.5"
            />
            <span class="ml-0.5 text-[11px] text-zinc-500">%</span>
          </div>
        {:else if isCustom}
          <button
            class={cn(
              "flex items-center gap-0.5 px-1.5 py-0.5 text-[11px] font-medium rounded transition-all",
              "bg-accent/20 text-accent",
            )}
            onclick={handleCustomClick}
          >
            {value}%
            <svg
              class="w-2.5 h-2.5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z"
              />
            </svg>
          </button>
        {:else}
          <button
            class={cn(
              "p-0.5 text-zinc-500 hover:text-zinc-400 transition-all",
            )}
            onclick={handleCustomClick}
            title="Custom slippage"
            aria-label="Set custom slippage"
          >
            <svg
              class="w-3 h-3"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z"
              />
            </svg>
          </button>
        {/if}
      </div>
    </div>
  </div>
</div>
