export function new_event() {
  return new Promise(resolve => {
    window.addEventListener("test-event", resolve);
    window.dispatchEvent(new Event("test-event", {
      bubbles: true,
      cancelable: true,
      composed: true,
    }));
  });
}
