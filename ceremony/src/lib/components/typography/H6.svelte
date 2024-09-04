<script lang="ts">
  let {children, class: className = '', ...props} = $props()
</script>

<h6 class={`text-union-heading-primary text-base font-bold font-supermolot mb-1.5 ${className}`} {...props}>
  {@render children()}
</h6>
