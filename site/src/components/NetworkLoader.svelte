<script>
  export let visible = true;
</script>

{#if visible}
  <div id="global-loader" class="loader">
    <div class="logo-pulse">
      <!-- Inline SVG logo -->
      <svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 48 48">
        <path fill="#5865F2" d="M24 0C10.745 0 0 10.745 0 24s10.745 24 24 24 24-10.745 24-24S37.255 0 24 0zm12 32h-8v8h-8v-8h-8v-8h8v-8h8v8h8v8z"/>
      </svg>
    </div>
  </div>
{/if}

<style>
  .loader {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
    background: #0f172a;
    z-index: 9999;
  }
  
  .logo-pulse {
    animation: pulse 2s infinite;
  }
  
  @keyframes pulse {
    0% { transform: scale(1); opacity: 0.9; }
    50% { transform: scale(1.05); opacity: 0.7; }
    100% { transform: scale(1); opacity: 0.9; }
  }
</style>
