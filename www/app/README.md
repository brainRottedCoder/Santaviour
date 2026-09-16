# Santa overlay (export-safe)

Custom ranking UI lives in this folder so `turbo export` can regenerate
`www/index.html`, `www/main.js`, `www/style.css`, and `www/pkg/` without
deleting the prototype.

After every `turbo export`, restore one line in `www/index.html` (before `main.js`):

```html
<script type="module" src="./app/ui.js"></script>
```

Do not put overlay CSS in `www/style.css`. `ui.js` injects `overlay.css`.

`www/admin.html` (with `admin.js` / `admin.css` here) is a standalone page that
`turbo export` does not touch, so it needs no restore step. It never loads the
WASM runtime. From the game overlay, Ctrl+Shift+A (Cmd+Shift+A on Mac) opens
`/admin`.

Verified against Turbo Genesis SDK 5.2.1:

- `events::emit(name, data)` is re-exported at the crate root (`pub use sys::*`)
  and dispatches `CustomEvent("turboGameEvent")`.
- `pause()` / `resume()` are exported from `www/pkg/turbo_genesis_impl_wasm_bindgen.js`.

`bridge.js` listens for `turboGameEvent` and also watches existing `log!`
lines so runs still sync if `www/main.turbo` has not been rebuilt yet.
