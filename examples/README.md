# AppJS Examples

This folder is a Bun workspace package in the AppJS monorepo.

## Install workspace deps

From repo root:

```bash
bun install
```

## Build examples to dist/

From repo root:

```bash
bun run build:examples
```

Or from this folder:

```bash
bun run build
```

## Run examples with AppJS runtime

From repo root:

```bash
bun run run:counter
bun run run:styled-counter
bun run run:test-ui
bun run run:test-simple
```

Or from this folder:

```bash
bun run run:counter
bun run run:counter-ts
bun run run:solid-apis
bun run run:styled-counter
bun run run:test-ui
bun run run:test-simple
```

These commands bundle the selected example with Bun into `examples/dist` and then run:

```bash
cargo run -- <bundled-script>
```
