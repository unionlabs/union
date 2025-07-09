<script>
  export let src;
  export let alt = "";
  export let width;
  export let height;
  export let lazy = true;
  
  $: optimizedSrc = src.replace('/public/', '/public/optimized/');
</script>

<img 
  src={optimizedSrc} 
  alt={alt}
  width={width}
  height={height}
  loading={lazy ? 'lazy' : 'eager'}
  decoding="async"
/>
