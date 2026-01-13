# Contributing to CORE-FFX

Thanks for contributing to CORE-FFX. This guide covers local setup, workflow, and conventions.

## Getting Started

### Prerequisites

- Node.js 18+
- Rust (stable toolchain via rustup)
- Tauri dependencies for your OS

### Clone

```bash
git clone https://github.com/YOUR_USERNAME/AD1-tools.git
cd AD1-tools
git remote add upstream https://github.com/CORE/AD1-tools.git
```

## Development Setup

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

## Cleaning

```bash
npm run clean
npm run clean:all
```

## Testing

- Rust: `cargo test` (run from `src-tauri/`)
- Frontend: no dedicated test runner is configured yet

## Workflow

- Create a branch per change
- Keep commits focused and descriptive
- Use conventional commit messages when possible (`feat`, `fix`, `docs`, `refactor`)

## Coding Standards

### TypeScript / SolidJS

- Prefer typed props and state
- Use SolidJS reactive primitives (`createSignal`, `createMemo`, `createEffect`)
- Keep components small and focused

### Rust

- Follow standard Rust idioms
- Avoid `unwrap()` in production paths
- Add doc comments for public APIs

## Documentation

If a change alters behavior or APIs, update the relevant docs. Start here:

- `README.md`
- `APP_README.md`
- `CODE_BIBLE.md`
- `src-tauri/src/README.md`

## Extensions

See `src/extensions/README.md` for extension development guidance.
