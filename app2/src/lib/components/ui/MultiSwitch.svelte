<script
  lang="ts"
  generics="K, T extends { key: K, label: string, disabled?: boolean | undefined, class?: string | undefined }[]"
>
import { cn } from "$lib/utils"

type Change = {
  key: K
  option: T[number]
  index: number
}

type Props = {
  options: T
  selectedKey: K
  class?: string
  ariaLabel?: string | undefined
  orientation: "horizontal" | "vertical"
  onChange: (e: Change) => void
}

let {
  selectedKey = $bindable<K>(),
  onChange,
  options,
  orientation,
  ariaLabel = "Choose option",
  class: className = "",
}: Props = $props()

$effect(() => {
  if (!options?.length) {
    return
  }
  const has = selectedKey != null && options.some(o => o.key === selectedKey)
  if (!has) {
    const first = options.find(o => !o.disabled)
    if (first) {
      selectedKey = first.key
    }
  }
})

function select(i: number) {
  const opt = options[i]
  if (!opt || opt.disabled) {
    return
  }
  if (selectedKey === opt.key) {
    return
  }

  selectedKey = opt.key
  onChange({ key: opt.key, option: opt, index: i })
}
</script>

<div
  role="group"
  aria-label={ariaLabel}
  class={cn(
    "inline-flex",
    orientation === "horizontal" ? "flex-row gap-0" : "flex-col gap-0",
    className,
  )}
>
  {#each options as opt, i (opt.key)}
    <button
      type="button"
      aria-pressed={selectedKey === opt.key}
      disabled={opt.disabled}
      class={cn(
        "grow px-2 py-1 text-xs font-mono border transition-colors min-h-[32px] focus:outline-none focus-visible:ring-2 focus-visible:ring-zinc-500/60",
        orientation === "horizontal"
          ? "-ml-px first:ml-0 first:rounded-l last:rounded-r"
          : "-mt-px first:mt-0 first:rounded-t last:rounded-b",
        selectedKey === opt.key
          ? "border-zinc-500 bg-zinc-800 text-zinc-200 font-medium"
          : "border-zinc-700 bg-zinc-900 text-zinc-400 hover:border-zinc-600 hover:text-zinc-300",
        opt.disabled && "opacity-50 cursor-not-allowed",
        opt.class,
      )}
      onclick={() => select(i)}
    >
      {opt.label}
    </button>
  {/each}
</div>
