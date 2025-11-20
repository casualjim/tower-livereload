(() => {
  const inputs = document.currentScript.dataset;

  addEventListener("pageshow", () => {
    const source = new EventSource(inputs.eventStream);
    let hasConnected = false;

    source.addEventListener("init", () => {
      // If we've already connected before, this is a reconnection after error
      // In that case, reload the page to get the latest content
      if (hasConnected) {
        source.close();
        window.location.reload();
      }
      // Mark that we've successfully connected
      hasConnected = true;
    });

    source.addEventListener("reload", () => {
      source.close();
      window.location.reload();
    });
  });
})();
