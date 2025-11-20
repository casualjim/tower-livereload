(() => {
  const inputs = document.currentScript.dataset;
  
  const RELOAD_COOLDOWN_MS = 1000; // Don't reload more than once per second

  addEventListener("pageshow", () => {
    const source = new EventSource(inputs.eventStream);
    let serverInstanceId = null;

    source.addEventListener("init", (event) => {
      const newInstanceId = event.data;
      
      // If we have a previous instance ID and it differs from the new one,
      // the server has restarted - reload the page
      if (serverInstanceId !== null && serverInstanceId !== newInstanceId) {
        const now = Date.now();
        const lastReloadTime = parseInt(sessionStorage.getItem('lr_last_reload') || '0', 10);
        
        // Only reload if we haven't reloaded recently (prevents reload loops during rapid restarts)
        if (now - lastReloadTime >= RELOAD_COOLDOWN_MS) {
          sessionStorage.setItem('lr_last_reload', String(now));
          source.close();
          window.location.reload();
        }
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
