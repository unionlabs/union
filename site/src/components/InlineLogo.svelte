<script>
  export let size = "md";
  
  const sizes = {
    sm: { width: 24, height: 24 },
    md: { width: 40, height: 40 },
    lg: { width: 64, height: 64 }
  };
  
  $: dimensions = sizes[size] || sizes.md;
</script>

<svg 
  xmlns="http://www.w3.org/2000/svg" 
  width={dimensions.width} 
  height={dimensions.height} 
  viewBox="0 0 48 48"
  aria-label="Union Labs Logo"
>
  <!-- Optimized Union Logo -->
  <path fill="#5865F2" fill-rule="evenodd" d="M24 0C10.745 0 0 10.745 0 24s10.745 24 24 24 24-10.745 24-24S37.255 0 24 0zm12 32h-8v8h-8v-8h-8v-8h8v-8h8v8h8v8z"/>
</svg>
