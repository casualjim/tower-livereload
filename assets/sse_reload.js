(() => {
  const inputs = document.currentScript.dataset;

  addEventListener("pageshow", () => {
    const source = new EventSource(inputs.eventStream);
    source.addEventListener("reload", () => {
      source.close();
      window.location.reload();
    });
  });
})();
