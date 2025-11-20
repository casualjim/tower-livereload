import { test, expect } from "@playwright/test";

test("no reload loop - stable connection", async ({ page }) => {
  await page.goto("/");

  // Expect heading to ensure we are looking at the intended page.
  await expect(
    page.getByRole("heading", { name: "Playwright!" }),
  ).toBeVisible();

  // Track navigation/reload events AFTER the initial load
  let reloadCount = 0;
  page.on("framenavigated", (frame) => {
    if (frame === page.mainFrame()) {
      reloadCount++;
    }
  });

  // Wait for 5 seconds and ensure no unexpected reloads happen
  await page.waitForTimeout(5000);

  // We expect 0 additional navigations after the initial page load
  expect(reloadCount).toBe(0);
});

test("no reload loop - reconnection simulation", async ({ page }) => {
  await page.goto("/");

  // Expect heading to ensure we are looking at the intended page.
  await expect(
    page.getByRole("heading", { name: "Playwright!" }),
  ).toBeVisible();

  // Track navigation/reload events AFTER the initial load
  let reloadCount = 0;
  page.on("framenavigated", (frame) => {
    if (frame === page.mainFrame()) {
      reloadCount++;
    }
  });

  // Simulate a network interruption by evaluating JavaScript that closes and reopens EventSource
  // This simulates what happens when the network briefly disconnects but server hasn't restarted
  await page.evaluate(() => {
    // Find the EventSource by looking for it in the global scope
    // The script should create it when pageshow event fires
    const oldEventSource = (window as any).eventSource;
    
    // Close the existing connection
    if (oldEventSource) {
      oldEventSource.close();
    }
    
    // Wait a bit, then reconnect to the SAME server instance
    // This simulates a network glitch where connection is lost and restored
    setTimeout(() => {
      const script = document.querySelector('script[data-event-stream]');
      if (script) {
        const eventStream = (script as HTMLElement).dataset.eventStream;
        if (eventStream) {
          const source = new EventSource(eventStream);
          (window as any).eventSource = source;
          
          let serverInstanceId: string | null = null;
          
          source.addEventListener("init", (event: any) => {
            const newInstanceId = event.data;
            
            if (serverInstanceId !== null && serverInstanceId !== newInstanceId) {
              source.close();
              window.location.reload();
              return;
            }
            
            serverInstanceId = newInstanceId;
          });
          
          source.addEventListener("reload", () => {
            source.close();
            window.location.reload();
          });
        }
      }
    }, 100);
  });

  // Wait for a bit to ensure reconnection happens
  await page.waitForTimeout(3000);

  // We expect 0 additional navigations after the initial page load
  // No reload should happen because the server instance ID is the same
  expect(reloadCount).toBe(0);
});

test("verify single reload on manual trigger", async ({ page, request }) => {
  await page.goto("/");

  // Expect heading to ensure we are looking at the intended page.
  await expect(
    page.getByRole("heading", { name: "Playwright!" }),
  ).toBeVisible();

  // Track navigation/reload events AFTER the initial load
  let reloadCount = 0;
  page.on("framenavigated", (frame) => {
    if (frame === page.mainFrame()) {
      reloadCount++;
    }
  });

  // Wait for the reload to start
  const reloadPromise = page.waitForEvent("framenavigated");

  // Trigger a reload via the API
  await request.post("/reload");

  // Wait for reload to complete
  await reloadPromise;

  // Wait a bit more to ensure no additional reloads happen
  await page.waitForTimeout(2000);

  // We expect exactly 1 additional navigation (the triggered reload)
  expect(reloadCount).toBe(1);

  // Verify we're still on the page after reload
  await expect(
    page.getByRole("heading", { name: "Playwright!" }),
  ).toBeVisible();
});
