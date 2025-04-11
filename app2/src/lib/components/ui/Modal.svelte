<script lang="ts">
  import type { Snippet } from "svelte";
  import { onMount } from "svelte";
  import Card from "./Card.svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { cn } from "$lib/utils";
  import { fade, scale } from "svelte/transition";

  type Props = HTMLAttributes<HTMLDivElement> & {
    children: Snippet;
    isOpen: boolean;
    onClose: () => void;
    class?: string;
    showCloseButton?: boolean;
    divided?: boolean;
  };

  const {
    children,
    isOpen,
    onClose,
    class: className = "",
    showCloseButton = true,
    divided = false,
    ...rest
  }: Props = $props();

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && isOpen) {
      onClose();
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }

  onMount(() => {
    document.addEventListener("keydown", handleKeydown);
    return () => {
      document.removeEventListener("keydown", handleKeydown);
    };
  });
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div
    class="fixed inset-0 flex items-center justify-center z-50 w-screen h-screen bg-black/90"
    onclick={handleBackdropClick}
    role="dialog"
    aria-modal="true"
    transition:fade={{ duration: 100 }}
  >
    <div
      class="relative flex"
      transition:scale={{ duration: 100, start: 0.55 }}
    >
      <Card
        transition={false}
        class={cn(
          "flex-1 h-full max-h-[600px] min-h-[375px] max-w-md relative flex flex-col z-20",
          className,
        )}
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
  </div>
{/if}
