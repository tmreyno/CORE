# Contributing to CORE-FFX

Thanks for contributing to CORE-FFX — the **C**ase **O**rganization & **R**eporting **E**cosystem forensic file explorer. This guide covers local setup, workflow, and conventions.

## Code of Conduct

Be respectful and constructive. We follow standard open-source community guidelines. Harassment, discrimination, and personal attacks are not tolerated.

## Getting Started

### Prerequisites

- Node.js 18+
- Rust (stable toolchain via rustup)
- Tauri dependencies for your OS

### Clone

```bash
git clone https://github.com/tmreyno/CORE.git
cd CORE
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

```bash
# Rust tests
cd src-tauri && cargo test

# Frontend tests
npx vitest              # Watch mode
npx vitest --run        # Single run (CI)
```

All PRs should pass existing tests. Add tests for new features when feasible.

## Workflow

1. Fork or create a branch per change
2. Keep commits focused and descriptive
3. Use conventional commit messages (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`)
4. Open a pull request with a clear description of the change
5. Ensure CI checks pass before requesting review

## Issues

- Search existing issues before opening a new one
- Include reproduction steps, environment details, and expected behavior
- Use labels when available (bug, enhancement, documentation)

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

If a change alters behavior or APIs, update the relevant docs:

- `README.md` — Project overview and feature list
- `CODE_BIBLE.md` — Authoritative codebase map
- `.github/copilot-instructions.md` — AI coding agent instructions
- `CRATE_API_NOTES.md` — Third-party crate API reference
- `FRONTEND_API_NOTES.md` — SolidJS/TypeScript API reference
- `src-tauri/src/README.md` — Backend module reference
- `src/components/README.md` — Frontend component catalog
- `src/hooks/README.md` — State management hooks
