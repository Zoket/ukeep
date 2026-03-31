# AGENTS.md — uKeep Development Guide

uKeep is a Dioxus (Rust/WASM) web application for tracking food inventory with expiration dates.
Always answer in Simplified Chinese.

## Project Structure

```
src/
├── main.rs          # Entry point, App component, global state setup
├── lib.rs           # Module exports
├── router.rs        # Route definitions (Dioxus router)
├── state.rs         # Global state (InventoryState with Signal<Vec<Item>>)
├── storage.rs       # LocalStorage persistence, import/export
├── models/
│   ├── mod.rs
│   └── item.rs      # Item struct with business logic
├── pages/
│   ├── mod.rs
│   ├── home.rs      # Main inventory list view
│   └── add_item.rs  # Add new item form
├── components/
│   ├── mod.rs
│   └── item_card.rs # Swipeable card component
└── utils.rs         # Mock data generator
```

---

## Build / Lint / Test Commands

### Rust (Primary)

```bash
# Development server (web platform)
dx serve

# Specific platform
dx serve --platform desktop
dx serve --platform mobile

# Standard Cargo commands also work
cargo check
cargo build
cargo build --release

# Lint with clippy
cargo clippy

# Format code
cargo fmt
```

### Tailwind CSS

```bash
# Watch mode for development
npx tailwindcss -i ./tailwind.css -o ./assets/tailwind.css --watch

# One-time build
npx tailwindcss -i ./tailwind.css -o ./assets/tailwind.css --minify
```

### Docker (Production)

```bash
# Local deploy
./docker-deploy.sh

# Docker Compose
docker-compose up -d
```

### Testing

**No test framework currently configured.** Add tests as needed:

```bash
cargo test
```

---

## Code Style Guidelines

### Rust General

- Follow standard Rust conventions (rustfmt default)
- Use `clippy.toml` rules: avoid holding GenerationalRef/Write across await points
- Error handling: use `Result<T, E>` with `map_err` for context; use `log::error!`/`log::warn!` for failures
- Avoid `unwrap()` in production code; prefer `?` operator or `match`

### Dioxus-Specific

- **Component naming**: PascalCase (e.g., `Home`, `ItemCard`) — Dioxus convention
- **Global state**: Use `use_context_provider` with `Signal` (see `state.rs`)
- **Signal usage**: Clone signals when capturing in closures; avoid holding `GenerationalRef` across await
- **Event handlers**: Use `EventHandler<T>` for callbacks; call with `.call(value)`
- **Conditional rendering**: `if condition { rsx!(...) }` pattern
- **Loops in rsx**: `for item in items { rsx!(...) }` with `key:` attribute

### Module Organization

- `lib.rs`: Public module exports (`pub mod`, `pub use`)
- `mod.rs`: Submodule barrel exports per directory
- Private implementation details stay in child modules

### Imports

```rust
use dioxus::prelude::*;        // Dioxus macros and types
use crate::modules::Item;      // Internal imports via crate::
```

### Frontend (Tailwind + rsx!)

- Tailwind CSS v4 with `mode: "all"`
- Use Material Symbols Outlined for icons: `<span class="material-symbols-outlined">icon_name</span>`
- Chinese UI text throughout the app
- CSS classes in rsx!: `class: "tailwind-classes"`
- Dynamic styles: `style: "property: {value};"` or `style: format!(...)`

### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Modules | snake_case | `item_card.rs` |
| Structs/Enums | PascalCase | `struct Item`, `enum Route` |
| Functions | snake_case | `save_inventory()` |
| Variables | snake_case | `inventory`, `error_message` |
| Components | PascalCase | `Home`, `ItemCard` |
| Constants | SCREAMING_SNAKE_CASE | `SWIPE_THRESHOLD` |

### Error Handling Patterns

```rust
// Storage errors: log + return default/empty
match LocalStorage::get::<Vec<Item>>(KEY) {
    Ok(items) => items,
    Err(e) => {
        log::warn!("Failed to load: {:?}", e);
        Vec::new()
    }
}

// File/blob operations: Result with context
LocalStorage::set(KEY, items).map_err(|_| "Save failed".to_string())?
```

### State Management

- Global state via `InventoryState(pub Signal<Vec<Item>>)` in `state.rs`
- Component-local state via `use_signal(|| initial_value)`
- State mutations: `inventory.write().push(item)` / `inventory.write().retain(...)`

---

## Key Dependencies

| Package | Purpose |
|---------|---------|
| `dioxus` | UI framework with router, signals, web support |
| `dioxus-signals` | Reactive state management |
| `gloo-storage` | LocalStorage wrapper |
| `gloo-file` | File read/write for import/export |
| `serde` / `serde_json` | Serialization |
| `chrono` | Date/time handling (NaiveDate) |
| `uuid` | Unique item IDs |
| `wasm-bindgen` | JS interop |

---

## Platform Notes

- **Web (WASM)**: Primary target; uses LocalStorage, Service Worker for PWA
- **Desktop/Mobile**: Feature flags `desktop` / `mobile` in Cargo.toml
- WASM optimization disabled (`wasm_opt level = "0"`) to avoid DWARF issues
- Release profile optimized for size: `opt-level = "z"`, LTO enabled, panic = "abort"

---

## Common Patterns

### Reading State in Effects

```rust
use_effect(move || {
    let items = inventory.read().clone();
    save_inventory(&items);
});
```

### Async Web Operations

```rust
wasm_bindgen_futures::spawn_local(async move {
    // async work...
    let promise = navigator.register("/sw.js");
    match JsFuture::from(promise).await {
        Ok(_) => log::info!("Success"),
        Err(e) => log::error!("Failed: {:?}", e),
    }
});
```

### JS Interop

```rust
use web_sys::{Window, Document, HtmlElement};
use wasm_bindgen::JsCast;

if let Some(window) = web_sys::window() {
    let navigator = window.navigator();
    // ...
}
```
