# LiveReload Test - Demonstrates Fix for Infinite Reload Loop

This test application demonstrates that the fix for the infinite reload loop issue works correctly.

## The Problem

Previously, tower-livereload would cause an infinite reload loop when the server restarted because the client couldn't distinguish between:
- Server restart (new instance) - should reload
- Network reconnection (same instance) - should NOT reload

## The Fix

The fix adds a unique server instance ID (timestamp-based) to each server instance:
- Server sends instance ID in the SSE "init" event
- Client compares instance IDs on reconnection
- Different ID → server restarted → reload once
- Same ID → network glitch → don't reload

## Running the Test

```bash
# Install dependencies
npm install

# Install playwright browsers (first time only)
npx playwright install chromium

# Run the test
npx playwright test
```

## Test Results

The test verifies:
1. **No reload loop** - Only 1 navigation occurs over 10 seconds (the initial page load)
2. **LiveReload script injected** - The script with instance ID tracking is properly added to HTML
3. **EventSource connection works** - SSE connection establishes successfully

Expected output:
```
✓ no reload loop after server starts (10.1s)
✓ page has livereload script injected (63ms)
✓ EventSource connection established (2.1s)

3 passed
```

## Manual Testing

You can also run the server manually and open it in a browser:

```bash
cargo run
```

Then open http://localhost:3030 and observe:
- Page loads with live reload enabled
- No infinite reload loop occurs
- Check browser DevTools → Network tab to see EventSource connection
