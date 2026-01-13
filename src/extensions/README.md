# CORE-FFX Extension Development Guide

This guide describes the extension registry and available extension categories.

## Extension Categories

- database-processor
- container-parser
- artifact-viewer
- export-format
- analysis-tool
- integration
- theme
- utility

## Manifest

```ts
interface ExtensionManifest {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  category: ExtensionCategory;
  license?: string;
  homepage?: string;
  repository?: string;
  minAppVersion?: string;
  keywords?: string[];
  icon?: string;
}
```

## Registration

```ts
import { registerExtension, enableExtension } from "@core-ffx/extensions";

await registerExtension(myExtension);
await enableExtension(myExtension.manifest.id);
```

## Notes

- Extensions run inside the application process and are not sandboxed.
- For now, extensions are registered at runtime in the UI; there is no packaging pipeline.
- See `src/extensions/registry.ts` for lifecycle details.

## Examples

`src/extensions/examples/timeline-viewer.tsx`
