import { test, expect } from "@playwright/test";

test("homepage loads without reload loop", async ({ page }) => {
  // Track navigation events
  let navigationCount = 0;
  page.on("framenavigated", (frame) => {
    if (frame === page.mainFrame()) {
      navigationCount++;
      console.log(`Navigation ${navigationCount} detected`);
    }
  });

  // Navigate to the homepage
  await page.goto("/");

  // Verify page loaded
  await expect(page).toHaveTitle(/dainty-test/);

  // Wait 15 seconds to ensure no reload loop occurs
  console.log("Waiting 15 seconds to verify no reload loop...");
  await page.waitForTimeout(15000);

  // We expect exactly 1 navigation (the initial load)
  // If there's a reload loop, we'd see multiple navigations
  console.log(`Total navigations: ${navigationCount}`);
  expect(navigationCount).toBe(1);
  
  console.log("✓ No reload loop detected! The fix works in dainty project.");
});

test("livereload script is injected", async ({ page }) => {
  await page.goto("/");
  
  // Check that the livereload script is injected
  const scripts = await page.evaluate(() => {
    const scripts = Array.from(document.querySelectorAll('script'));
    return scripts.map(s => ({
      hasEventStream: s.dataset.eventStream !== undefined,
      eventStream: s.dataset.eventStream,
      content: s.innerHTML.substring(0, 100)
    }));
  });
  
  const liveReloadScript = scripts.find(s => s.hasEventStream);
  expect(liveReloadScript).toBeTruthy();
  expect(liveReloadScript?.eventStream).toContain('/_tower-livereload/event-stream');
  
  console.log("✓ LiveReload script with instance ID tracking properly injected in dainty project");
});
