<script lang="ts">
  let {children, class: className = '', ...props} = $props()
</script>

<h5 class={`text-union-heading-primary text-lg font-bold font-supermolot ${className}`} {...props}>
  {@render children()}
</h5>
