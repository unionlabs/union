<script lang="ts">
  import { onMount } from "svelte"
  import Card from "./Card.svelte"
  import type { HTMLAttributes } from "svelte/elements"
  import { cn } from "$lib/utils"
  import type { Snippet } from "svelte"

  type Props = HTMLAttributes<HTMLDivElement> & {
    children: Snippet
    isOpen: boolean
    onClose: () => void
    class?: string
    showCloseButton?: boolean
    divided?: boolean
  }

  const { 
    children,
    isOpen,
    onClose,
    class: className = "",
    showCloseButton = true,
    divided = false,
    ...rest
  }: Props = $props()

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onClose()
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose()
    }
  }

  onMount(() => {
    document.addEventListener("keydown", handleKeydown)
    return () => {
      document.removeEventListener("keydown", handleKeydown)
    }
  })
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div 
    class="fixed inset-0 bg-black/90 flex items-center justify-center z-50"
    onclick={handleBackdropClick}
    role="dialog"
    aria-modal="true"
  >
    <Card 
      class={cn("max-h-[600px] min-h-[375px] h-full w-full max-w-md relative flex flex-col z-20", className)}
      {divided}
      {...rest}
    >
      {#if showCloseButton}
        <button
          class="cursor-pointer border-0 absolute top-2 right-4 text-white text-lg"
          onclick={onClose}
        >
          ✕
        </button>
      {/if}
      {@render children()}
    </Card>
  </div>
{/if}
