<script lang="ts">
  let {children, class: CLASS, ...props} = $props()
</script>

<a {...props} class={`text-union-text-primary hover:underline underline-offset-4 decoration-union-accent-500 ${CLASS}`}>
  {@render children()}
</a>