(() => {
  const inputs = document.currentScript.dataset;

  addEventListener("pageshow", () => {
    const source = new EventSource(inputs.eventStream);
    let serverInstanceId = null;

    source.addEventListener("init", (event) => {
      const newInstanceId = event.data;
      
      // If we have a previous instance ID and it differs from the new one,
      // the server has restarted - reload the page
      if (serverInstanceId !== null && serverInstanceId !== newInstanceId) {
        source.close();
        window.location.reload();
        return;
      }
      
      // Store the current server instance ID
      serverInstanceId = newInstanceId;
    });

    source.addEventListener("reload", () => {
      source.close();
      window.location.reload();
    });
  });
})();
