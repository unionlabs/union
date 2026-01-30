<script lang="ts">
/**
 * Generates a pixelated grayscale pattern from block hash
 * Like a mosaic/censor blur effect
 */

interface Props {
  hash: string
  size?: number
  gridSize?: number
  class?: string
}

const { hash, size = 140, gridSize = 10, class: className = "" }: Props = $props()

// Convert hash to array of numbers
function hashToNumbers(hash: string): number[] {
  const clean = hash.replace("0x", "").toUpperCase()
  const numbers: number[] = []
  for (let i = 0; i < clean.length; i += 2) {
    numbers.push(parseInt(clean.slice(i, i + 2), 16) || 0)
  }
  return numbers
}

// Generate grayscale grid from hash
function generateGrid(nums: number[], gridSize: number): number[][] {
  const grid: number[][] = []

  for (let y = 0; y < gridSize; y++) {
    const row: number[] = []
    for (let x = 0; x < gridSize; x++) {
      const idx = (y * gridSize + x) % nums.length
      // Map to grayscale value (0-255)
      row.push(nums[idx])
    }
    grid.push(row)
  }

  return grid
}

const nums = $derived(hashToNumbers(hash))
const grid = $derived(generateGrid(nums, gridSize))
const cellSize = $derived(size / gridSize)

function getCellColor(value: number): string {
  // Grayscale with some contrast adjustment
  const v = Math.floor(value * 0.6 + 40) // Range ~40-190 for better contrast
  return `rgb(${v}, ${v}, ${v})`
}
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 {size} {size}"
  class="block-visual {className}"
>
  {#each grid as row, y}
    {#each row as cell, x}
      <rect
        x={x * cellSize}
        y={y * cellSize}
        width={cellSize}
        height={cellSize}
        fill={getCellColor(cell)}
      />
    {/each}
  {/each}
</svg>

<style>
  .block-visual {
    border-radius: 2px;
  }
</style>
