export function trackPerf() {
  if (typeof window !== 'undefined') {
    window.addEventListener('load', () => {
      const timing = performance.timing;
      const loadTime = timing.loadEventEnd - timing.navigationStart;
      
      if (loadTime > 5000) {
        console.warn(`[PERF] Slow load detected: ${loadTime}ms`);
        // Send to analytics: analytics.track('slow_load', { loadTime })
      }
    });
  }
}
