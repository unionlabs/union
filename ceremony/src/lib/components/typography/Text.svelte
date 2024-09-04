<script lang="ts">
  let { children, class: CLASS, ...props } = $props()
</script>

<p class={`text-union-text-primary leading-6 font-mono ${CLASS}`} {...props}>
  {@render children()}
</p>
