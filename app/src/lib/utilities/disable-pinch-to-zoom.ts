
/// Attempts to prevent pinch-to-zoom. Still does not work in all cases.
export const disablePinchToZoom = () => {
  document.addEventListener('touchmove', (event) => {
  	// @ts-ignore
    if (event.scale !== 1) { event.preventDefault(); }
  }, false);

  document.addEventListener("gesturestart", (e) => {
  	e.preventDefault();
    	// @ts-ignore
      document.body.style.zoom = 1;
  });

  document.addEventListener("gesturechange", (e) => {
  	e.preventDefault();

  	// @ts-ignore
    document.body.style.zoom = 1;
  });
  document.addEventListener("gestureend", (e) => {
  	  e.preventDefault();
    	// @ts-ignore
      document.body.style.zoom = 1;
  });

}
