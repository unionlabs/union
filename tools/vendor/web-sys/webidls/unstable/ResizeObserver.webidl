enum ResizeObserverBoxOptions {
    "border-box", "content-box", "device-pixel-content-box"
};

dictionary ResizeObserverOptions {
    ResizeObserverBoxOptions box = "content-box";
};

[Exposed=(Window)]
interface ResizeObserver {
    constructor(ResizeObserverCallback callback);
    undefined observe(Element target, optional ResizeObserverOptions options = {});
    undefined unobserve(Element target);
    undefined disconnect();
};

callback ResizeObserverCallback = undefined (sequence<ResizeObserverEntry> entries, ResizeObserver observer);

[Exposed=Window]
interface ResizeObserverEntry {
    readonly attribute Element target;
    readonly attribute DOMRectReadOnly contentRect;
    readonly attribute FrozenArray<ResizeObserverSize> borderBoxSize;
    readonly attribute FrozenArray<ResizeObserverSize> contentBoxSize;
    readonly attribute FrozenArray<ResizeObserverSize> devicePixelContentBoxSize;
};

[Exposed=Window]
interface ResizeObserverSize {
    readonly attribute unrestricted double inlineSize;
    readonly attribute unrestricted double blockSize;
};
