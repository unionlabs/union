export function inView(
  node: HTMLElement,
  parameters: Record<string, any> = {}
) {
  const { once = false, threshold = 0, rootMargin = "0px" } = parameters;

  const observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting) {
          node.dispatchEvent(new CustomEvent("in-view", { detail: entry }));
          if (once) {
            observer.unobserve(node);
          }
        }
      }
    },
    { threshold, rootMargin }
  );

  observer.observe(node);

  return {
    destroy() {
      observer.unobserve(node);
    },
  };
}
