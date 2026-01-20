# Transfer Module Refactoring

The `TransferPanel.tsx` (1157 lines) has been modularized into smaller, focused components:

## New Structure

```
src/components/transfer/
├── index.ts              # Barrel export
├── types.ts              # Type definitions (ContainerType, FileTreeNode, TransferJob, etc.)
├── utils.ts              # Utility functions (detectContainerType, buildFileTree, etc.)
├── FileTreeNode.tsx      # Tree node component with icons and progress
├── TransferOptions.tsx   # Options panel (verify, hash, timestamps, etc.)
├── SourceList.tsx        # Source files/folders list with container badges
└── TransferJobCard.tsx   # Individual job progress card
```

## Benefits

1. **Single Responsibility**: Each file has one clear purpose
2. **Testability**: Smaller units are easier to test
3. **Reusability**: Components can be used elsewhere
4. **Maintainability**: Changes isolated to relevant files
5. **Code Navigation**: Easier to find and understand code

## Usage

Import from the barrel export:
```tsx
import { 
  FileTreeNodeComponent,
  TransferOptions,
  SourceList,
  TransferJobCard,
  detectContainerType,
  buildFileTree,
  type TransferJob,
  type FileTreeNode,
} from "./transfer";
```

## Migration

The main `TransferPanel.tsx` can now import from `./transfer` instead of defining everything inline. This reduces the main file to ~400-500 lines of orchestration logic.

## Future Improvements

Consider similar refactoring for:
- `EvidenceTreeLazy.tsx` (1949 lines) → `tree/` module
- `ReportWizard.tsx` (2040 lines) → `report/wizard/` steps
- `SettingsPanel.tsx` (1105 lines) → `settings/` sections
- `App.tsx` (1378 lines) → Extract panel renderers, hooks
