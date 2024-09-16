<script lang="ts">
let { children, class: className = "", ...props } = $props()
</script>

<h1 class={`text-union-text-primary text-3xl font-bold font-supermolot uppercase  ${className}`} {...props}>
  {@render children()}
</h1>
