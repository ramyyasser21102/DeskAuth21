// jsdom lacks ResizeObserver, which Radix uses to place tooltips and select menus.
class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}

globalThis.ResizeObserver ??= ResizeObserverStub;
