<script lang="ts">
  let {children, class: CLASS, ...props } = $props()
</script>

<div class={`text-xs text-union-accent-500 bg-union-accent-500/20 border border-union-accent-500 uppercase px-2 py-1`}>
  {@render children()}
</div>