<script lang="ts">
  let {children, class: className = '', ...props} = $props()
</script>

<a {...props} class={`text-union-text-primary hover:underline underline-offset-4 decoration-union-accent-500 ${className}`}>
  {@render children()}
</a>