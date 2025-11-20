# HTMZ Standalone Demo - Explanation

## What is HTMZ?

**HTMZ (HTML with Zero server)** is a pattern for building web applications that work entirely in the browser without requiring a backend server. It uses:

- **Data URLs** for dynamic content generation
- **iframes** for content swapping and updates
- **HTML forms** for all interactions
- **Browser storage** (localStorage, IndexedDB) as the database

## Two Demo Versions

### 1. `index.html` - Standard Demo
**When to use:** Most users, GitHub Pages deployment, development

```
Browser → ES Modules → WASM
         ↓
    Simple fetch()
```

**Features:**
- Requires HTTP server (GitHub Pages, `python3 -m http.server`, etc.)
- Standard ES module imports
- Clean, familiar patterns
- Easy for contributors to understand

**To run:**
```bash
python3 -m http.server 8080
# Visit: http://localhost:8080/www/
```

### 2. `htmz-standalone.html` - HTMZ Pattern
**When to use:** Offline demos, distribution, teaching, research

```
Browser ← No server needed!
   ↓
Everything self-contained
```

**Features:**
- ✅ Works completely offline (file:// protocol)
- ✅ No server setup required
- ✅ Double-click to open
- ✅ Can be emailed as single file (with pkg/)
- ✅ Demonstrates browser-as-server pattern
- ✅ HTMZ form-driven architecture

**To run:**
```bash
# Option 1: Just double-click the file!
# Option 2: Or still use a server
python3 -m http.server 8080
# Visit: http://localhost:8080/www/htmz-standalone.html
```

## Technical Differences

### Standard Demo (`index.html`)
```html
<script type="module">
    import init, { parse_ixml } from '../pkg/rustixml.js';
    
    // Direct button onclick handlers
    document.getElementById('parse-btn').onclick = () => {
        // Parse and update DOM
    };
</script>
```

### HTMZ Demo (`htmz-standalone.html`)
```html
<!-- Form-driven interactions -->
<form action="#output-display" target="htmz" onsubmit="return parseInput(this)">
    <button type="submit">Parse</button>
</form>

<!-- Hidden iframe for content updates -->
<iframe name="htmz" onload="htmzUpdate(this)"></iframe>

<script type="module">
    // HTMZ handler updates targets via iframe
    window.htmzUpdate = function(iframe) {
        const targetId = iframe.src.split('#')[1];
        const content = iframe.contentDocument.body.innerHTML;
        document.getElementById(targetId).innerHTML = content;
    };
</script>
```

## Key HTMZ Concepts Demonstrated

### 1. Form-Driven Actions
Instead of `onClick` handlers, uses HTML forms:
```html
<form action="#target" target="htmz" onsubmit="return handler(this)">
```

### 2. Fragment Identifiers for Targets
The `#target` in the action URL specifies where content should be injected.

### 3. Hidden iframe for State
The `<iframe name="htmz">` serves as a communication channel between form submissions and content updates.

### 4. No Server Routes Required
All "routing" happens in the browser via fragment identifiers and event handlers.

## When to Use Each Version

### Use Standard Demo (`index.html`) when:
- Publishing to GitHub Pages
- Contributing to the project
- Learning iXML (not HTMZ)
- Building production apps
- Want familiar patterns

### Use HTMZ Demo (`htmz-standalone.html`) when:
- Demonstrating offline capability
- Teaching browser-as-server patterns
- Distributing to users without server access
- Research on serverless architectures
- Want to email a working demo

## File Structure

Both demos require the same WASM files:

```
rustixml/
├── www/
│   ├── index.html              ← Standard (most users)
│   └── htmz-standalone.html    ← HTMZ pattern (offline/research)
└── pkg/                         ← WASM files (shared by both)
    ├── rustixml.js
    ├── rustixml_bg.wasm
    └── rustixml.d.ts
```

## Performance Comparison

| Metric | Standard | HTMZ |
|--------|----------|------|
| Load Time | ~100ms | ~100ms |
| Parse Speed | Same | Same |
| Offline Work | ❌ No | ✅ Yes |
| Server Required | ✅ Yes | ❌ No |
| Code Complexity | Lower | Slightly Higher |

## Future HTMZ Features (Not Yet Implemented)

The HTMZ standalone demo is a minimal proof-of-concept. The full HTMZ pattern could include:

1. **Real-time Updates** - iframe polling for live data
2. **Browser Storage** - IndexedDB for persistent grammars
3. **Data URLs** - Dynamic content generation without files
4. **Routing System** - Multiple "pages" via fragments
5. **Form Validation** - HTML5 native validation
6. **Template System** - Reusable HTML fragments

## Learning Resources

- **HTMZ Specification**: `/home/bigale/repos/miniz-cssz-studio/browserchain/HTMZ-SERVER-GRAMMAR.md`
- **iXML Specification**: https://invisiblexml.org/
- **rustixml Docs**: `../docs/`

## Conclusion

Both demos showcase rustixml's WASM capabilities. The standard demo is recommended for most users, while the HTMZ demo demonstrates advanced browser-as-server patterns and works completely offline.

Choose based on your use case:
- **Learning iXML?** → Use `index.html`
- **Offline/research?** → Use `htmz-standalone.html`
- **Production app?** → Use `index.html` as template
- **Teaching HTMZ?** → Use `htmz-standalone.html` as example
