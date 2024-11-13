<script lang="ts">
  interface Props {
    children?: import('svelte').Snippet;
  }

  let { children }: Props = $props();
</script>

<h2 class="text-lg text-foreground font-bold font-supermolot">
  {@render children?.()}
</h2>
