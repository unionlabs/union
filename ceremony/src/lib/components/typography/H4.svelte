<script lang="ts">
let { children, class: className = "", ...props } = $props()
</script>

<h4 class={`text-union-heading-primary text-xl font-bold font-supermolot uppercase ${className}`} {...props}>
  {@render children()}
</h4>
