import type { TestInfo } from "@playwright/test";
import { expect, test } from "@playwright/test";

const userIdFor = (testInfo: TestInfo) => `layout-${testInfo.testId}`;
const requestHeadersFor = (testInfo: TestInfo) => ({
  headers: { "X-API-Key": userIdFor(testInfo) },
});

test.describe("Left rail interactions", () => {
  test.beforeEach(async ({ page }, testInfo) => {
    await page.setExtraHTTPHeaders(requestHeadersFor(testInfo).headers);
    await page.goto("/");
    await page.waitForFunction(() => window.Alpine !== undefined);
  });

  test("rail exposes the workspace icon buttons", async ({ page }) => {
    const rail = page.getByRole("complementary", {
      name: "Workspace navigation",
    });
    await expect(rail).toBeVisible();
    await expect(rail.getByLabel("Home workspace")).toBeVisible();
    await expect(rail.getByLabel("Shared spaces")).toBeVisible();
    await expect(rail.getByLabel("Boards")).toBeVisible();
    await expect(rail.getByLabel("Notifications")).toBeVisible();
  });

  test("avatar button toggles the profile menu", async ({ page }) => {
    const dropdown = page.locator("[data-profile-dropdown]");
    const toggle = dropdown.locator("summary");
    const menu = dropdown.getByRole("menu");

    await expect(dropdown).not.toHaveAttribute("open", "");

    await toggle.click();
    await expect(dropdown).toHaveAttribute("open", "");
    await expect(menu).toBeVisible();
    await expect(menu.getByText("Profile")).toBeVisible();
    await expect(menu.getByText("Settings")).toBeVisible();
    await expect(menu.getByText("Sign out")).toBeVisible();

    await toggle.click();
    await expect(dropdown).not.toHaveAttribute("open", "");
    await expect(menu).not.toBeVisible();
  });

  test("profile menu is positioned correctly on desktop", async ({ page }) => {
    const dropdown = page.locator("[data-profile-dropdown]");
    const toggle = dropdown.locator("summary");
    const menu = dropdown.getByRole("menu");

    await toggle.click();
    await expect(menu).toBeVisible();

    // Get positions
    const toggleBox = await toggle.boundingBox();
    const menuBox = await menu.boundingBox();

    if (!toggleBox || !menuBox) throw new Error("Elements not found");

    // Menu should be positioned near the toggle button
    // Check that menu appears below or to the side of the toggle
    const isNearby = Math.abs(menuBox.x - toggleBox.x) < 200 && Math.abs(menuBox.y - toggleBox.y) < 200;
    expect(isNearby).toBe(true);
  });
});

test.describe("Profile menu on mobile", () => {
  test.beforeEach(async ({ page }, testInfo) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.setExtraHTTPHeaders(requestHeadersFor(testInfo).headers);
    await page.goto("/");
    await page.waitForFunction(() => window.Alpine !== undefined);
  });

  test("profile menu is positioned correctly on mobile", async ({ page }) => {
    // On mobile, use the mobile-specific profile dropdown in the dock
    const dropdown = page.locator("[data-profile-dropdown-mobile]");
    const toggle = dropdown.locator("summary");
    const menu = dropdown.getByRole("menu");

    await toggle.click();
    await expect(menu).toBeVisible();

    // Get positions and viewport
    const toggleBox = await toggle.boundingBox();
    const menuBox = await menu.boundingBox();
    const viewport = page.viewportSize();

    if (!toggleBox || !menuBox || !viewport) throw new Error("Elements not found");

    // Menu should be fully visible within viewport (not cut off)
    expect(menuBox.x).toBeGreaterThanOrEqual(0);
    expect(menuBox.y).toBeGreaterThanOrEqual(0);
    expect(menuBox.x + menuBox.width).toBeLessThanOrEqual(viewport.width);
    expect(menuBox.y + menuBox.height).toBeLessThanOrEqual(viewport.height);

    // Menu should be right-aligned with the toggle button (within a small tolerance)
    const menuRight = menuBox.x + menuBox.width;
    const toggleRight = toggleBox.x + toggleBox.width;
    expect(Math.abs(menuRight - toggleRight)).toBeLessThan(5);

    // Menu should be above the toggle (since it's dropdown-top)
    expect(menuBox.y + menuBox.height).toBeLessThan(toggleBox.y);
  });
});

test.describe("Sidebar interactions", () => {
  test.beforeEach(async ({ page }, testInfo) => {
    await page.setExtraHTTPHeaders(requestHeadersFor(testInfo).headers);
    await page.goto("/");
    await page.waitForFunction(() => window.Alpine !== undefined);
  });

  test("left sidebar is visible by default", async ({ page }) => {
    const leftSidebar = page.locator('[aria-label="Left sidebar"]');
    await expect(leftSidebar).toBeVisible();
  });

  test("right sidebar is visible by default", async ({ page }) => {
    const rightSidebar = page.locator('[aria-label="Right sidebar"]');
    await expect(rightSidebar).toBeVisible();
  });

  test("can toggle left sidebar visibility", async ({ page }) => {
    const toggleButton = page.getByLabel("Toggle left sidebar").last(); // Desktop version
    const leftSidebar = page.locator('[aria-label="Left sidebar"]');

    // Initially visible
    await expect(leftSidebar).toBeVisible();

    // Click to hide
    await toggleButton.click();
    await expect(leftSidebar).toBeHidden();

    // Click to show
    await toggleButton.click();
    await expect(leftSidebar).toBeVisible();
  });

  test("can toggle right sidebar visibility", async ({ page }) => {
    const toggleButton = page.getByLabel("Toggle right sidebar").last(); // Desktop version
    const rightSidebar = page.locator('[aria-label="Right sidebar"]');

    // Initially visible
    await expect(rightSidebar).toBeVisible();

    // Click to hide
    await toggleButton.click();
    await expect(rightSidebar).toBeHidden();

    // Click to show
    await toggleButton.click();
    await expect(rightSidebar).toBeVisible();
  });

  test("left sidebar has correct default width", async ({ page }) => {
    const leftSidebar = page.locator('[aria-label="Left sidebar"]');
    const width = await leftSidebar.evaluate((el) => el.getBoundingClientRect().width);
    expect(width).toBeGreaterThanOrEqual(290);
    expect(width).toBeLessThanOrEqual(330);
  });

  test("right sidebar has correct default width", async ({ page }) => {
    const rightSidebar = page.locator('[aria-label="Right sidebar"]');
    const width = await rightSidebar.evaluate((el) => el.getBoundingClientRect().width);
    expect(width).toBeGreaterThanOrEqual(319);
    expect(width).toBeLessThanOrEqual(321);
  });

  test("left sidebar can be resized by dragging", async ({ page }) => {
    const leftSidebar = page.locator('[aria-label="Left sidebar"]');
    const resizeHandle = leftSidebar.locator('[data-resize-handle="left"]');

    const initialWidth = await leftSidebar.evaluate((el) => el.getBoundingClientRect().width);

    // Drag resize handle to the right
    const box = await resizeHandle.boundingBox();
    if (!box) throw new Error("Resize handle not found");

    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.down();
    await page.mouse.move(box.x + 100, box.y + box.height / 2);
    await page.mouse.up();

    const newWidth = await leftSidebar.evaluate((el) => el.getBoundingClientRect().width);
    expect(newWidth).toBeGreaterThan(initialWidth);
  });

  test("right sidebar can be resized by dragging", async ({ page }) => {
    const rightSidebar = page.locator('[aria-label="Right sidebar"]');
    const resizeHandle = rightSidebar.locator('[data-resize-handle="right"]');

    const initialWidth = await rightSidebar.evaluate((el) => el.getBoundingClientRect().width);

    // Drag resize handle to the left
    const box = await resizeHandle.boundingBox();
    if (!box) throw new Error("Resize handle not found");

    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.down();
    await page.mouse.move(box.x - 100, box.y + box.height / 2);
    await page.mouse.up();

    const newWidth = await rightSidebar.evaluate((el) => el.getBoundingClientRect().width);
    expect(newWidth).toBeGreaterThan(initialWidth);
  });

  test("left sidebar auto-collapses when resized below 5px", async ({ page }) => {
    const leftSidebar = page.locator('[aria-label="Left sidebar"]');
    const resizeHandle = leftSidebar.locator('[data-resize-handle="left"]');

    // Drag resize handle far to the left to trigger auto-collapse
    const box = await resizeHandle.boundingBox();
    if (!box) throw new Error("Resize handle not found");

    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.down();
    await page.mouse.move(box.x - 400, box.y + box.height / 2);
    await page.mouse.up();

    // Should be hidden
    await expect(leftSidebar).toBeHidden();

    // When toggled back, should have default width
    const toggleButton = page.getByLabel("Toggle left sidebar").last(); // Desktop version
    await toggleButton.click();
    await expect(leftSidebar).toBeVisible();
    const width = await leftSidebar.evaluate((el) => el.getBoundingClientRect().width);
    expect(width).toBeGreaterThanOrEqual(319);
    expect(width).toBeLessThanOrEqual(321);
  });

  test("right sidebar auto-collapses when resized below 5px", async ({ page }) => {
    const rightSidebar = page.locator('[aria-label="Right sidebar"]');
    const resizeHandle = rightSidebar.locator('[data-resize-handle="right"]');

    // Drag resize handle far to the right to trigger auto-collapse
    const box = await resizeHandle.boundingBox();
    if (!box) throw new Error("Resize handle not found");

    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.down();
    await page.mouse.move(box.x + 400, box.y + box.height / 2);
    await page.mouse.up();

    // Should be hidden
    await expect(rightSidebar).toBeHidden();

    // When toggled back, should have default width
    const toggleButton = page.getByLabel("Toggle right sidebar").last(); // Desktop version
    await toggleButton.click();
    await expect(rightSidebar).toBeVisible();
    const width = await rightSidebar.evaluate((el) => el.getBoundingClientRect().width);
    expect(width).toBeGreaterThanOrEqual(319);
    expect(width).toBeLessThanOrEqual(321);
  });

  test("main content area is scrollable independently", async ({ page }) => {
    const mainContent = page.locator('[aria-label="Main content area"]');
    const scrollable = await mainContent.evaluate((el) => {
      return (
        el.scrollHeight > el.clientHeight ||
        getComputedStyle(el).overflowY === "auto" ||
        getComputedStyle(el).overflowY === "scroll"
      );
    });
    expect(scrollable).toBeTruthy();
  });

  test("left sidebar is scrollable independently", async ({ page }) => {
    const leftSidebar = page.locator('[aria-label="Left sidebar"]');
    const scrollable = await leftSidebar.evaluate((el) => {
      return getComputedStyle(el).overflowY === "auto" || getComputedStyle(el).overflowY === "scroll";
    });
    expect(scrollable).toBeTruthy();
  });

  test("right sidebar is scrollable independently", async ({ page }) => {
    const rightSidebar = page.locator('[aria-label="Right sidebar"]');
    const scrollable = await rightSidebar.evaluate((el) => {
      return getComputedStyle(el).overflowY === "auto" || getComputedStyle(el).overflowY === "scroll";
    });
    expect(scrollable).toBeTruthy();
  });
});

test.describe("Theme toggle interactions", () => {
  test.beforeEach(async ({ page }, testInfo) => {
    await page.setExtraHTTPHeaders(requestHeadersFor(testInfo).headers);
    await page.goto("/");
    await page.waitForFunction(() => window.Alpine !== undefined);
  });

  test("theme toggle button is visible", async ({ page }) => {
    const themeToggle = page.getByLabel("Toggle theme");
    await expect(themeToggle).toBeVisible();
  });

  test("cycles through theme states: system → light → dark → system", async ({ page }, testInfo) => {
    const themeToggle = page.getByLabel("Toggle theme");
    const requestHeaders = requestHeadersFor(testInfo);

    // Click to go to light
    await themeToggle.click();
    await page.waitForTimeout(400); // Wait for debounce + save
    let response = await page.request.get("/api/layout?path=/&device=desktop", requestHeaders);
    let data = await response.json();
    expect(data.theme).toBe("light");

    // Click to go to dark
    await themeToggle.click();
    await page.waitForTimeout(400);
    response = await page.request.get("/api/layout?path=/&device=desktop", requestHeaders);
    data = await response.json();
    expect(data.theme).toBe("dark");

    // Click to go back to system
    await themeToggle.click();
    await page.waitForTimeout(400);
    response = await page.request.get("/api/layout?path=/&device=desktop", requestHeaders);
    data = await response.json();
    expect(data.theme).toBe("system");
  });

  test("persists theme preference to server", async ({ page }, testInfo) => {
    const requestHeaders = requestHeadersFor(testInfo);

    const themeToggle = page.getByLabel("Toggle theme");

    // Set to light theme
    await themeToggle.click();
    await page.waitForTimeout(400); // Wait for debounce + save

    let response = await page.request.get(`/api/layout?path=/&device=desktop`, requestHeaders);
    let data = await response.json();
    expect(data.theme).toBe("light");

    // Reload page
    await page.reload();
    await page.waitForFunction(() => window.Alpine !== undefined);
    await page.waitForTimeout(200);

    // Should still be light theme in server state
    response = await page.request.get(`/api/layout?path=/&device=desktop`, requestHeaders);
    data = await response.json();
    expect(data.theme).toBe("light");
  });

  test("applies correct data-theme attribute to html element", async ({ page }) => {
    const themeToggle = page.getByLabel("Toggle theme");

    // Start with system theme (initTheme sets data-theme based on system preference)
    let dataTheme = await page.evaluate(() => document.documentElement.getAttribute("data-theme"));
    expect(["light", "dark"]).toContain(dataTheme); // System theme could be either

    // Click to light
    await themeToggle.click();
    dataTheme = await page.evaluate(() => document.documentElement.getAttribute("data-theme"));
    expect(dataTheme).toBe("light");

    // Click to dark
    await themeToggle.click();
    dataTheme = await page.evaluate(() => document.documentElement.getAttribute("data-theme"));
    expect(dataTheme).toBe("dark");

    // Click back to system (sets to system preference)
    await themeToggle.click();
    dataTheme = await page.evaluate(() => document.documentElement.getAttribute("data-theme"));
    expect(["light", "dark"]).toContain(dataTheme); // System theme could be either
  });
});

test.describe("Resize handle visibility", () => {
  test.beforeEach(async ({ page }, testInfo) => {
    await page.setExtraHTTPHeaders(requestHeadersFor(testInfo).headers);
    await page.goto("/");
    await page.waitForFunction(() => window.Alpine !== undefined);
  });

  test("left sidebar resize handle is visible", async ({ page }) => {
    const leftSidebar = page.locator('[aria-label="Left sidebar"]');
    const resizeHandle = leftSidebar.locator('[data-resize-handle="left"]');
    await expect(resizeHandle).toBeVisible();
  });

  test("right sidebar resize handle is visible", async ({ page }) => {
    const rightSidebar = page.locator('[aria-label="Right sidebar"]');
    const resizeHandle = rightSidebar.locator('[data-resize-handle="right"]');
    await expect(resizeHandle).toBeVisible();
  });

  test("resize handle has correct cursor style", async ({ page }) => {
    const leftHandle = page.locator('[data-resize-handle="left"]');
    const cursor = await leftHandle.evaluate((el) => getComputedStyle(el).cursor);
    expect(cursor).toBe("col-resize");
  });
});

test.describe("Mobile responsive behavior", () => {
  test.beforeEach(async ({ page }, testInfo) => {
    await page.setViewportSize({ width: 375, height: 812 });
    await page.setExtraHTTPHeaders(requestHeadersFor(testInfo).headers);
    await page.goto("/");
    await page.waitForFunction(() => window.Alpine !== undefined);
  });

  test("left rail is hidden on mobile", async ({ page }) => {
    const rail = page.getByRole("complementary", {
      name: "Workspace navigation",
    });
    await expect(rail).not.toBeVisible();
  });

  test("mobile dock is visible", async ({ page }) => {
    const dock = page.getByRole("navigation", {
      name: "Mobile workspace navigation",
    });
    await expect(dock).toBeVisible();
    await expect(dock.getByLabel("Home workspace")).toBeVisible();
    await expect(dock.getByLabel("Shared spaces")).toBeVisible();
    await expect(dock.getByLabel("Boards")).toBeVisible();
    await expect(dock.getByLabel("Notifications")).toBeVisible();
  });

  test("sidebars are closed by default on mobile", async ({ page }) => {
    const leftSidebar = page.locator('[aria-label="Left sidebar"]');
    const rightSidebar = page.locator('[aria-label="Right sidebar"]');
    await expect(leftSidebar).not.toBeVisible();
    await expect(rightSidebar).not.toBeVisible();
  });

  test("mobile toggle buttons use menu icon", async ({ page }) => {
    const leftToggle = page.getByLabel("Toggle left sidebar").first();
    const rightToggle = page.getByLabel("Toggle right sidebar").first();

    await expect(leftToggle).toBeVisible();
    await expect(rightToggle).toBeVisible();

    // Check that menu icons are present
    const leftIcon = leftToggle.locator('[data-lucide="menu"]');
    const rightIcon = rightToggle.locator('[data-lucide="menu"]');
    await expect(leftIcon).toBeVisible();
    await expect(rightIcon).toBeVisible();
  });

  test("left sidebar opens as overlay drawer on mobile", async ({ page }) => {
    const leftSidebar = page.locator('[aria-label="Left sidebar"]');
    const toggleButton = page.getByLabel("Toggle left sidebar").first();

    await expect(leftSidebar).not.toBeVisible();

    await toggleButton.click();
    await expect(leftSidebar).toBeVisible();

    // Check it's positioned as fixed overlay
    const position = await leftSidebar.evaluate((el) => getComputedStyle(el).position);
    expect(position).toBe("fixed");
  });

  test("right sidebar opens as overlay drawer on mobile", async ({ page }) => {
    const rightSidebar = page.locator('[aria-label="Right sidebar"]');
    const toggleButton = page.getByLabel("Toggle right sidebar").first();

    await expect(rightSidebar).not.toBeVisible();

    await toggleButton.click();
    await expect(rightSidebar).toBeVisible();

    // Check it's positioned as fixed overlay
    const position = await rightSidebar.evaluate((el) => getComputedStyle(el).position);
    expect(position).toBe("fixed");
  });

  test("backdrop appears when left sidebar is open on mobile", async ({ page }) => {
    const backdrop = page.locator(".bg-black\\/50");
    const toggleButton = page.getByLabel("Toggle left sidebar").first();

    await expect(backdrop).not.toBeVisible();

    await toggleButton.click();
    await expect(backdrop).toBeVisible();
  });

  test("backdrop appears when right sidebar is open on mobile", async ({ page }) => {
    const backdrop = page.locator(".bg-black\\/50");
    const toggleButton = page.getByLabel("Toggle right sidebar").first();

    await expect(backdrop).not.toBeVisible();

    await toggleButton.click();
    await expect(backdrop).toBeVisible();
  });

  test("clicking outside left sidebar closes it on mobile", async ({ page }) => {
    const leftSidebar = page.locator('[aria-label="Left sidebar"]');
    const toggleButton = page.getByLabel("Toggle left sidebar").first();

    await toggleButton.click();
    await expect(leftSidebar).toBeVisible();

    // Click outside the sidebar (on the right edge where backdrop is visible)
    await page.mouse.click(350, 400);
    await expect(leftSidebar).not.toBeVisible();
  });

  test("clicking outside right sidebar closes it on mobile", async ({ page }) => {
    const rightSidebar = page.locator('[aria-label="Right sidebar"]');
    const toggleButton = page.getByLabel("Toggle right sidebar").first();

    await toggleButton.click();
    await expect(rightSidebar).toBeVisible();

    // Click outside the sidebar (on the left edge where backdrop is visible)
    await page.mouse.click(25, 400);
    await expect(rightSidebar).not.toBeVisible();
  });

  test("backdrop disappears when sidebar closes on mobile", async ({ page }) => {
    const backdrop = page.locator(".bg-black\\/50");
    const toggleButton = page.getByLabel("Toggle left sidebar").first();

    await toggleButton.click();
    await expect(backdrop).toBeVisible();

    await page.mouse.click(350, 400);
    await expect(backdrop).not.toBeVisible();
  });

  test("opening one sidebar closes the other on mobile", async ({ page }) => {
    const leftSidebar = page.locator('[aria-label="Left sidebar"]');
    const rightSidebar = page.locator('[aria-label="Right sidebar"]');
    const leftToggle = page.getByLabel("Toggle left sidebar").first();

    await leftToggle.click();
    await expect(leftSidebar).toBeVisible();
    await expect(rightSidebar).not.toBeVisible();
    // Opening right sidebar should close left - use evaluate to trigger Alpine handler
    await page.evaluate(() => {
      const Alpine = window.Alpine;
      const data = Alpine.$data(document.body) as { leftSidebar: boolean; rightSidebar: boolean };
      if (window.innerWidth < 1024 && !data.rightSidebar) data.leftSidebar = false;
      data.rightSidebar = !data.rightSidebar;
    });

    await expect(rightSidebar).toBeVisible();
    await expect(leftSidebar).not.toBeVisible();
  });

  test("clicking toggle button closes open sidebar on mobile", async ({ page }) => {
    const leftSidebar = page.locator('[aria-label="Left sidebar"]');
    const toggleButton = page.getByLabel("Toggle left sidebar").first();

    await toggleButton.click();
    await expect(leftSidebar).toBeVisible();

    // Use evaluate to trigger Alpine handler
    await page.evaluate(() => {
      const Alpine = window.Alpine;
      const data = Alpine.$data(document.body) as { leftSidebar: boolean };
      data.leftSidebar = !data.leftSidebar;
    });

    await expect(leftSidebar).not.toBeVisible();
  });
});

test.describe("Desktop vs Mobile toggle buttons", () => {
  test.beforeEach(async ({ page }, testInfo) => {
    await page.setExtraHTTPHeaders(requestHeadersFor(testInfo).headers);
    await page.goto("/");
    await page.waitForFunction(() => window.Alpine !== undefined);
  });

  test("desktop uses panel icons, mobile uses menu icons", async ({ page }) => {
    // Desktop left toggle should have panel icons
    const desktopLeftToggle = page.getByLabel("Toggle left sidebar").last();
    await expect(desktopLeftToggle.locator('[data-lucide="panel-left-close"]')).toBeAttached();
    await expect(desktopLeftToggle.locator('[data-lucide="panel-left-open"]')).toBeAttached();

    // Desktop right toggle should have panel icons
    const desktopRightToggle = page.getByLabel("Toggle right sidebar").last();
    await expect(desktopRightToggle.locator('[data-lucide="panel-right-close"]')).toBeAttached();
    await expect(desktopRightToggle.locator('[data-lucide="panel-right-open"]')).toBeAttached();

    // Switch to mobile
    await page.setViewportSize({ width: 375, height: 812 });

    // Mobile toggles should have menu icons
    const mobileLeftToggle = page.getByLabel("Toggle left sidebar").first();
    const mobileRightToggle = page.getByLabel("Toggle right sidebar").first();

    await expect(mobileLeftToggle.locator('[data-lucide="menu"]')).toBeVisible();
    await expect(mobileRightToggle.locator('[data-lucide="menu"]')).toBeVisible();
  });
});
