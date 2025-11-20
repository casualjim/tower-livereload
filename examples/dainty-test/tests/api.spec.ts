import type { TestInfo } from "@playwright/test";
import { expect, test } from "@playwright/test";

const userIdFor = (testInfo: TestInfo) => `test-${testInfo.testId}`;

test.describe("API endpoints", () => {
  test("GET /api/layout returns stored settings only", async ({ request }, testInfo) => {
    const userId = userIdFor(testInfo);
    const initialResponse = await request.get("/api/layout?path=/", {
      headers: { "X-API-Key": userId },
    });

    expect(initialResponse.ok()).toBeTruthy();
    expect(initialResponse.status()).toBe(200);

    const initialData = await initialResponse.json();
    expect(initialData).toEqual({});

    await request.post("/api/layout", {
      headers: {
        "Content-Type": "application/json",
        "X-API-Key": userId,
      },
      data: {
        path: "/",
        device: "desktop",
        left_sidebar_open: false,
        theme: "dark",
      },
    });

    const response = await request.get("/api/layout?path=/&device=desktop", {
      headers: { "X-API-Key": userId },
    });

    expect(response.ok()).toBeTruthy();
    expect(response.status()).toBe(200);

    const data = await response.json();
    expect(data.left_sidebar_open).toBe(false);
    expect(data.theme).toBe("dark");
    expect(data).not.toHaveProperty("id");
    expect(data).not.toHaveProperty("user_id");
    expect(data).not.toHaveProperty("context_key");
  });

  test("POST /api/layout updates layout state", async ({ request }, testInfo) => {
    const userId = userIdFor(testInfo);
    // First, get current state (we don't use it here, but ensure endpoint responds)
    await request.get("/api/layout?path=/test&device=desktop", {
      headers: { "X-API-Key": userId },
    });

    // Update the state
    const updatePayload = {
      path: "/test",
      device: "desktop",
      left_sidebar_open: false,
      left_width: 400,
    };

    const postResponse = await request.post("/api/layout", {
      headers: {
        "Content-Type": "application/json",
        "X-API-Key": userId,
      },
      data: updatePayload,
    });

    expect(postResponse.ok()).toBeTruthy();
    expect(postResponse.status()).toBe(200);

    const updatedData = await postResponse.json();
    expect(updatedData.left_sidebar_open).toBe(false);
    expect(updatedData.left_width).toBe(400);

    // Verify the change persisted
    const verifyResponse = await request.get("/api/layout?path=/test&device=desktop", {
      headers: { "X-API-Key": userId },
    });
    const verifiedData = await verifyResponse.json();
    expect(verifiedData.left_sidebar_open).toBe(false);
    expect(verifiedData.left_width).toBe(400);
  });

  test("Different paths have different contexts", async ({ request }, testInfo) => {
    const userId = userIdFor(testInfo);
    // Update state for path /page1
    await request.post("/api/layout", {
      headers: {
        "Content-Type": "application/json",
        "X-API-Key": userId,
      },
      data: { path: "/page1", left_width: 500 },
    });

    // Update state for path /page2
    await request.post("/api/layout", {
      headers: {
        "Content-Type": "application/json",
        "X-API-Key": userId,
      },
      data: { path: "/page2", left_width: 600 },
    });

    // Verify they are different
    const page1Response = await request.get("/api/layout?path=/page1", {
      headers: { "X-API-Key": userId },
    });
    const page1Data = await page1Response.json();
    expect(page1Data.left_width).toBe(500);

    const page2Response = await request.get("/api/layout?path=/page2", {
      headers: { "X-API-Key": userId },
    });
    const page2Data = await page2Response.json();
    expect(page2Data.left_width).toBe(600);
  });

  test("POST with invalid JSON returns 400", async ({ request }, testInfo) => {
    const userId = userIdFor(testInfo);
    const response = await request.post("/api/layout", {
      headers: {
        "Content-Type": "application/json",
        "X-API-Key": userId,
      },
      data: "invalid json",
    });

    // This should be a 400; failing test indicates server returns incorrect status
    expect(response.status()).toBe(400);
  });

  test("Unsupported method returns 405 with Allow header", async ({ request }, testInfo) => {
    const userId = userIdFor(testInfo);
    const response = await request.delete("/api/layout", {
      headers: { "X-API-Key": userId },
    });

    expect(response.status()).toBe(405);
    expect(response.headers().allow).toBe("GET, POST");
  });
});
