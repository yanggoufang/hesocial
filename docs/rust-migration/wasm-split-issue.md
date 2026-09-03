# `--wasm-split` silently empties dynamic route segments, rewriting the URL

**Versions:** dioxus 0.7.10, dioxus-cli (`dx`) 0.7.10, wasm-split-cli 0.7.10, target `wasm32-unknown-unknown`, Chrome headless.

## Summary

With the `wasm-split` feature enabled and `dx bundle --wasm-split`, every route that carries a **dynamic segment** resolves to the wrong component. The browser URL itself is rewritten with the segment's value emptied, so the app then matches whatever shorter route the truncated path happens to hit — or no route at all.

Routes with no dynamic segment are unaffected. The same source built **without** `--wasm-split` behaves correctly.

## Reproduction

A `Routable` enum with both static and dynamic routes under one layout:

```rust
#[derive(Routable, Clone, PartialEq, Debug)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Shell)]
        #[route("/events")]
        Events {},
        #[route("/events/:id/participants")]
        EventParticipants { id: String },
        #[route("/events/:id")]
        EventDetail { id: String },
        #[route("/vvip")]
        Vvip {},
        #[route("/event-mgmt/media/:event_id")]
        EventMedia { event_id: String },
}
```

Build:

```
dx bundle --release --platform web --features <feature enabling dioxus/wasm-split> --wasm-split
```

(with `[profile.release]` carrying `lto = true` and `debug = true`)

Then hard-navigate to each URL:

| Navigated to | URL after load | Component rendered |
|---|---|---|
| `/events/11` | `/events` | `Events` |
| `/events/11/participants` | `/events/participants` | `EventDetail` with `id = "participants"` |
| `/event-mgmt/media/11` | `/event-mgmt/media` | none — `RouteMatchError` page |
| `/vvip` | `/vvip` | `Vvip` (correct) |

The chunk actually fetched matches the *wrong* component, i.e. `/events/11` fetches `module_N_routeEvents…wasm`. So the wrong decision is made before the lazy loader is consulted.

## What this is not

- **Not the JS glue.** Every `__wasm_split_load_module<Name><hash>_<hash>_route<Name><hash>` export in the bindgen output maps to the correct `module_N_route<Name>…wasm` URL. All were checked by hand.
- **Not wasm-opt.** `--debug-symbols false` (DWARF stripped) and the default (DWARF kept) behave identically.
- **Not the chunks' contents.** Loading `module_N_routeEventDetail` renders `EventDetail`; loading `module_M_routeEvents` renders `Events`. The split output itself is fine.
- **Not `dioxus-router-macro` codegen.** Expanding the crate with and without the `wasm-split` feature (`RUSTC_BOOTSTRAP=1 cargo rustc --lib --target wasm32-unknown-unknown -- -Zunpretty=expanded`) yields a **byte-identical** `impl std::fmt::Display for Route`, and it is correct:

  ```rust
  Self::EventDetail { id } => {
      f.write_fmt(format_args!("/{0}", "events"))?;
      {
          let as_string = id.to_string();
          f.write_fmt(format_args!("/{0}",
              dioxus_router::exports::percent_encoding::utf8_percent_encode(
                  &as_string, dioxus_router::exports::PATH_ASCII_SET)))?;
      }
  }
  ```

  The same generated code prints `/events/11` in an unsplit build and `/events/` in a split one.

## Workarounds tried, both still fail

Patching `dioxus-router-macro` locally via `[patch.crates-io]`:

1. Giving each route its own loader rather than sharing the `extern "<module>"` naming.
2. Removing `Route` from the split export's call graph entirely — the generated `fn route<Name><hash>` takes a `Box<(FieldTypes,)>` of just that route's field values, and `LazyLoader<Box<(…)>, Element>` replaces `LazyLoader<Route, Element>`, so the enum never crosses the `extern "C"` boundary.

Both produce the same emptied segments, which is consistent with the macro being innocent.

## Where I think the fault is

`Route`'s own methods are reachable from **both** the main module (parsing, `Display`, history canonicalization) and from every route chunk. That makes them shared symbols, and `wasm-split-cli`'s handling of shared symbols — `shared_symbols`, `delete_main_funcs_from_split`, `create_ifunc_initializers` in `wasm-split-cli/src/lib.rs` — is the remaining suspect. Note the failure mode is a **silently wrong result, not a trap**, which is what makes this dangerous well beyond routing: any code shared between the main module and a chunk could be affected the same way.

I have not isolated the exact function or table index; the evidence above is where I stopped.

## Impact

Route-level code splitting is unusable for any app with dynamic route segments. Everything else about the split works: measured on a 15-route snapshot of our app, the single wasm went from 345,988 gzipped to a 259,667 main chunk plus per-route chunks, so a landing-page visit dropped to 270,705. That is the only thing standing between us and a much smaller first paint, and we have had to ship unsplit at 765,738 instead.
