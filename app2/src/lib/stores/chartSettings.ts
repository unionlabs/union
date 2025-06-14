import { writable } from 'svelte/store'

// Shared chart settings
export const selectedItemCount = writable(3)

export const itemCounts = [
  { value: 3, label: '3' },
  { value: 4, label: '4' },
  { value: 7, label: '7' },
  { value: 10, label: '10' }
] 