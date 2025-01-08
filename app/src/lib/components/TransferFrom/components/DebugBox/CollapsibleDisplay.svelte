<script lang="ts">
interface Props {
  data: any
  label?: string
  depth?: number
  color?: string
  initiallyExpanded?: boolean
}

export let data: Props["data"]
export let label: Props["label"] = undefined
export let depth: Props["depth"] = 0
export let color: Props["color"] = "text-gray-200"
export let initiallyExpanded: Props["initiallyExpanded"] = false

let expanded = initiallyExpanded

function isObject(value: any): boolean {
  return value !== null && typeof value === "object" && !Array.isArray(value)
}

function formatValue(value: any): string {
  if (value === null) return "null"
  if (typeof value === "bigint") return value.toString()
  if (typeof value === "object") return ""
  return String(value)
}

function getValueClass(value: any): string {
  if (value === null) return "text-gray-500"
  if (typeof value === "boolean") return "text-purple-400"
  if (typeof value === "number") return "text-blue-400"
  if (typeof value === "string") return "text-green-400"
  return ""
}

function isEmpty(value: any): boolean {
  if (Array.isArray(value)) return value.length === 0
  if (isObject(value)) return Object.keys(value).length === 0
  return false
}

function getDisplayText(value: any): string {
  if (isEmpty(value)) return "No data available"
  if (Array.isArray(value)) return `Array(${value.length})`
  if (isObject(value)) return `Object{${Object.keys(value).length}}`
  return ""
}

function toggle() {
  expanded = !expanded
}
</script>

<div class="font-mono" style="margin-left: {depth * 1}rem">
  {#if label}
    <h3 class={color}>{label}:</h3>
  {/if}

  {#if data === null || data === undefined}
    <p class="text-sm text-gray-500">No data available</p>
  {:else if Array.isArray(data) || isObject(data)}
    <button
            class="flex items-center gap-2 cursor-pointer hover:bg-gray-800/50 px-2 py-1 rounded group"
            on:click|stopPropagation={toggle}
    >
      {#if !isEmpty(data)}
        <span class="text-gray-400 text-sm transition-transform duration-150 {expanded ? 'rotate-90' : ''} group-hover:text-gray-300">
          ▶
        </span>
      {/if}
      <span class="text-gray-400 text-sm group-hover:text-gray-300">
        {getDisplayText(data)}
      </span>
    </button>

    {#if !isEmpty(data)}
      <div
              class="overflow-hidden transition-[height,opacity] duration-150 ease-in-out"
              style="height: {expanded ? 'auto' : '0'}; opacity: {expanded ? '1' : '0'};"
      >
        {#if Array.isArray(data)}
          {#each data as item, i}
            <div class="ml-4 py-1">
              {#if isObject(item) || Array.isArray(item)}
                <div class="flex items-start gap-2">
                  <span class="text-gray-400 text-sm mt-1.5">{i}:</span>
                  <div class="flex-1">
                    <svelte:self
                            data={item}
                            depth={0}
                            {initiallyExpanded}
                    />
                  </div>
                </div>
              {:else}
                <div class="flex items-center gap-2">
                  <span class="text-gray-400">{i}:</span>
                  <span class={getValueClass(item)}>
                    {typeof item === 'string' ? `"${formatValue(item)}"` : formatValue(item)}
                  </span>
                </div>
              {/if}
            </div>
          {/each}
        {:else}
          {#each Object.entries(data) as [key, value]}
            <div class="ml-4 py-1">
              {#if isObject(value) || Array.isArray(value)}
                <div class="flex items-start gap-2">
                  <span class="text-gray-400 text-sm mt-1.5">{key}:</span>
                  <div class="flex-1">
                    <svelte:self
                            data={value}
                            depth={0}
                            {initiallyExpanded}
                    />
                  </div>
                </div>
              {:else}
                <div class="flex items-center gap-2">
                  <span class="text-gray-400">{key}:</span>
                  <span class={getValueClass(value)}>
                    {typeof value === 'string' ? `"${formatValue(value)}"` : formatValue(value)}
                  </span>
                </div>
              {/if}
            </div>
          {/each}
        {/if}
      </div>
    {/if}
  {:else}
    <p class="text-sm">
      <span class={getValueClass(data)}>
        {typeof data === 'string' ? `"${formatValue(data)}"` : formatValue(data)}
      </span>
    </p>
  {/if}
</div>