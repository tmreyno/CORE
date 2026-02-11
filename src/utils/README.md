# CORE-FFX Utilities

Shared frontend utility helpers.

## Files

```text
utils/
├── index.ts              # Barrel exports
├── accessibility.ts      # ARIA and keyboard accessibility helpers
├── containerUtils.ts     # Container type and format utilities
├── fileTypeUtils.ts      # File type detection and icon mapping
├── logger.ts             # Frontend logging utility
├── metadata.ts           # Metadata formatting helpers
├── operationProfiler.ts  # Operation timing and profiling
├── pathUtils.ts          # Path manipulation utilities
├── perfTestRunner.ts     # Performance test runner
├── performance.ts        # Performance monitoring
├── platform.ts           # Platform detection (macOS/Windows/Linux)
├── processed.ts          # Processed database utilities
└── telemetry.ts          # Telemetry and analytics
```

Also: `src/utils.ts` (root) — General formatting helpers (sizes, dates, type guards).

## Conventions

- Prefer pure functions
- Keep inputs/outputs typed
- Use small helpers rather than deep utility classes
