# File Protocol Limitation - Technical Note

## Issue

The HTMZ standalone demo requires an HTTP server and cannot work with `file://` protocol due to browser CORS restrictions on ES modules.

## Why This Happens

```
file:///path/to/htmz-standalone.html
  └─ tries to import from: file:///path/to/pkg/rustixml.js
                          ↓
                    🚫 BLOCKED by browser
                    (CORS policy for ES modules)
```

Browsers enforce CORS (Cross-Origin Resource Sharing) policies that prevent ES modules from loading via `file://` protocol for security reasons. This is by design and affects all modern browsers.

## Workarounds

### Option 1: Simple HTTP Server (Recommended)
```bash
# Python (built-in)
python3 -m http.server 8080

# Node.js
npx serve

# PHP
php -S localhost:8080

# Any other simple server works!
```

This is still "standalone" - no backend logic, no database, no API endpoints. The server just serves static files. All logic runs in the browser.

### Option 2: Inline WASM (Future Enhancement)

We could create a truly file:// compatible version by:

1. **Base64 encode the WASM binary** into the HTML
2. **Inline the JavaScript** glue code
3. **Load WASM from data URL**

Example approach:
```html
<script>
// Base64 encoded WASM (156KB → ~208KB base64)
const wasmBase64 = "AGFzbQEAAAA..."; // ~50,000 characters

// Decode and instantiate
const wasmBytes = Uint8Array.from(atob(wasmBase64), c => c.charCodeAt(0));
WebAssembly.instantiate(wasmBytes, imports).then(result => {
    // Parser ready!
});
</script>
```

**Trade-offs:**
- ✅ Works with file:// protocol
- ✅ True double-click to open
- ❌ HTML file becomes ~250KB (base64 overhead)
- ❌ Harder to maintain (binary in HTML)
- ❌ No browser caching benefits

### Option 3: Browser Extension

Package as a browser extension:
- ✅ Works offline
- ✅ No server needed
- ✅ Installable
- ❌ Requires extension store approval
- ❌ Different distribution model

### Option 4: Electron/Tauri App

Package as desktop app:
- ✅ Truly offline
- ✅ No CORS issues
- ✅ Native feel
- ❌ Much larger distribution
- ❌ Different development model

## Current Status

**Decision: Keep HTTP server requirement**

Reasoning:
1. **One-line command**: `python3 -m http.server 8080` is trivial
2. **Clean code**: Separate WASM file is maintainable
3. **Better performance**: Browser can cache WASM
4. **Standard practice**: Most WASM demos work this way
5. **HTMZ spirit**: Focus on no *backend* logic, not no server at all

The key HTMZ principle is **browser-as-server for logic**, not necessarily zero HTTP server for file serving. Once loaded, everything runs client-side.

## What "Standalone" Really Means

In HTMZ context, "standalone" means:

✅ **No backend code** - Python/Node/PHP/etc backend logic  
✅ **No database** - Uses browser storage  
✅ **No API calls** - Everything local  
✅ **No build step** - Just HTML/CSS/JS/WASM  
✅ **Portable** - Copy directory and run anywhere  

❌ **Not required:** Zero HTTP server (unrealistic for ES modules)

## Documentation Updates

Updated `www/HTMZ-README.md` to clarify:
- HTTP server is minimal and logic-free
- Browser CORS limitations explained
- Still demonstrates HTMZ pattern effectively
- Alternative approaches documented

## Future Consideration

If someone needs a truly file:// compatible version, we can create:
- `www/htmz-inline.html` - Base64 inlined WASM (250KB single file)
- Trade size for convenience
- Document as experimental/alternative approach

For now, the HTTP server requirement is acceptable and keeps the codebase clean.
