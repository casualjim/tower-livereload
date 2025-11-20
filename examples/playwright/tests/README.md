# Live Reload Integration Tests

This directory contains integration tests for the tower-livereload library, specifically focusing on preventing infinite reload loops.

## Problem Statement

Previously, the live reload implementation used a simple `hasConnected` flag to detect server restarts. This caused an infinite reload loop because:

1. Browser loads page, EventSource connects, receives "init", sets `hasConnected=true`
2. Server restarts
3. EventSource loses connection, tries to reconnect
4. EventSource reconnects to new server instance
5. New server sends "init" event
6. Client sees "init" but `hasConnected` is already true, so it reloads
7. Go back to step 1 - infinite loop!

## Solution

The fix adds a server instance ID (timestamp-based) to the SSE "init" event:
- Server generates a unique instance ID when it starts
- Client stores the server instance ID from the first "init" event
- On reconnection, if the instance ID differs, the server has restarted → reload
- If the instance ID is the same, it's just a network glitch → don't reload

## Test Coverage

### reload-loop.spec.ts

1. **no reload loop - stable connection**
   - Loads a page and waits 5 seconds
   - Verifies no unexpected reloads occur
   - Ensures the fix prevents reload loops during normal operation

2. **no reload loop - reconnection simulation**
   - Simulates EventSource disconnection and reconnection to the same server
   - Verifies the page does NOT reload (same instance ID)
   - Tests the network glitch scenario

3. **verify single reload on manual trigger**
   - Triggers a reload via the `/reload` API endpoint
   - Verifies exactly one reload occurs
   - Ensures manual reloads still work as expected

### simple.spec.ts

These are the original tests that verify basic functionality:

1. **has heading** - Verifies the page loads correctly
2. **reload** - Verifies manual reload triggers work
3. **no reload** (expected to fail) - Negative test for reload behavior

## Running the Tests

```bash
# Run all tests
npx playwright test

# Run only chromium tests
npx playwright test --project=chromium

# Run only reload loop tests
npx playwright test tests/reload-loop.spec.ts

# Run with UI mode (for debugging)
npx playwright test --ui
```

## Manual Verification

To manually verify the fix works:

1. Start the example server:
   ```bash
   cargo run --package playwright
   ```

2. Open http://localhost:3030 in your browser

3. Open the browser's developer console (F12)

4. Watch the Network tab for XHR/EventSource connections

5. Restart the server (Ctrl+C and run again)

6. You should see:
   - EventSource reconnects
   - Page reloads ONCE (due to new server instance ID)
   - No infinite reload loop

7. To test network glitch (no reload):
   - Run in console: `window.eventSource?.close()` (if you store it globally)
   - The page should NOT reload when EventSource reconnects to the same server
