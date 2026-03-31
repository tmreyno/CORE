#!/usr/bin/env bash
# =============================================================================
# CORE-FFX — Acquire Edition Boundary Lint
# Validates that Acquire-specific files only import from allowed paths.
# Run: bash scripts/lint-acquire-boundary.sh
# =============================================================================
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

ERRORS=0
WARNINGS=0

# ---------------------------------------------------------------------------
# Allowed sibling imports from src/components/acquire/ and src/hooks/acquire/
# These are shared components/utilities that WILL move to core-shared or a
# shared/ directory before the CORE-ACQ repo split.
# ---------------------------------------------------------------------------
ALLOWED_CROSS_BOUNDARY=(
  # Shared UI primitives (candidates for @core-suite/components)
  '../icons'
  '../Toast'
  '../viewerMetadata/shared'

  # Shared domain components (used by both editions)
  '../EvidenceCollectionPanel'
  '../EvidenceTree/containerDetection'
  '../RecentProjectsList'
  '../export-panel/DriveTreeBrowser'
  '../preferences'

  # Shared hooks/utilities
  '../export/companionHelper'
  '../project/useProjectDbSync'
)

# ---------------------------------------------------------------------------
# Review-only imports — these MUST NOT appear in acquire files
# ---------------------------------------------------------------------------
REVIEW_ONLY_PATTERNS=(
  'search/'
  'dedup/'
  'processed/'
  'report/'
  'workspace_profile'
  'project_comparison'
  'project_recovery'
  'project_templates'
  'activity_timeline'
  'SearchPanel'
  'SearchFilter'
  'DedupPanel'
  'ProcessedDb'
  'ReportWizard'
  'WorkspaceProfile'
)

echo "=== Acquire Edition Boundary Lint ==="
echo ""

# Step 1: Check for review-only imports in acquire files
echo "Step 1: Checking for review-only imports..."
for pattern in "${REVIEW_ONLY_PATTERNS[@]}"; do
  matches=$(grep -rn "from.*${pattern}" src/components/acquire/ src/hooks/acquire/ 2>/dev/null || true)
  if [ -n "$matches" ]; then
    echo -e "${RED}ERROR: Review-only import detected (pattern: ${pattern}):${NC}"
    echo "$matches"
    ERRORS=$((ERRORS + 1))
  fi
done

if [ "$ERRORS" -eq 0 ]; then
  echo -e "${GREEN}  ✓ No review-only imports found${NC}"
fi

echo ""

# Step 2: Catalog cross-boundary imports
echo "Step 2: Checking cross-boundary imports..."

# Get all non-relative, non-package imports from acquire files
cross_imports=$(grep -rn 'from "\.\.\/' src/components/acquire/ src/hooks/acquire/ 2>/dev/null \
  | grep -v 'from "\.\./\.\.' \
  | grep -v 'from "\./' \
  || true)

while IFS= read -r line; do
  [ -z "$line" ] && continue
  
  # Extract the import path
  import_path=$(echo "$line" | sed 's|.*from "\(\.\.\/[^"]*\)".*|\1|')
  file_loc=$(echo "$line" | cut -d: -f1-2)
  
  # Check if it's in the allowed list
  is_allowed=false
  for allowed in "${ALLOWED_CROSS_BOUNDARY[@]}"; do
    if [ "$import_path" = "$allowed" ]; then
      is_allowed=true
      break
    fi
  done
  
  if ! $is_allowed; then
    echo -e "${YELLOW}WARNING: Unregistered cross-boundary import:${NC}"
    echo "  ${file_loc}: ${import_path}"
    echo "  → Add to ALLOWED_CROSS_BOUNDARY or refactor before CORE-ACQ split"
    WARNINGS=$((WARNINGS + 1))
  fi
done <<< "$cross_imports"

if [ "$WARNINGS" -eq 0 ]; then
  echo -e "${GREEN}  ✓ All cross-boundary imports are registered${NC}"
fi

echo ""

# Step 3: Check that acquire hooks don't import from review-gated hooks
echo "Step 3: Checking hook boundary..."
review_hook_imports=$(grep -rn 'from.*hooks/.*search\|from.*hooks/.*dedup\|from.*hooks/.*report\|from.*hooks/.*processed' \
  src/hooks/acquire/ 2>/dev/null || true)

if [ -n "$review_hook_imports" ]; then
  echo -e "${RED}ERROR: Acquire hooks import from review-only hooks:${NC}"
  echo "$review_hook_imports"
  ERRORS=$((ERRORS + 1))
else
  echo -e "${GREEN}  ✓ No review-only hook imports${NC}"
fi

echo ""

# Step 4: Check API file boundary for acquire edition
echo "Step 4: Checking API imports in acquire files..."
review_api_imports=$(grep -rn 'from.*api/dedup\|from.*api/projectMerge\|from.*api/search' \
  src/components/acquire/ src/hooks/acquire/ 2>/dev/null || true)

if [ -n "$review_api_imports" ]; then
  echo -e "${RED}ERROR: Acquire files import review-only API modules:${NC}"
  echo "$review_api_imports"
  ERRORS=$((ERRORS + 1))
else
  echo -e "${GREEN}  ✓ No review-only API imports${NC}"
fi

echo ""

# Summary
echo "=== Summary ==="
if [ "$ERRORS" -gt 0 ]; then
  echo -e "${RED}✗ ${ERRORS} error(s) — boundary violations detected${NC}"
  if [ "$WARNINGS" -gt 0 ]; then
    echo -e "${YELLOW}  ${WARNINGS} warning(s) — unregistered cross-boundary imports${NC}"
  fi
  exit 1
elif [ "$WARNINGS" -gt 0 ]; then
  echo -e "${YELLOW}⚠ ${WARNINGS} warning(s) — unregistered cross-boundary imports${NC}"
  echo "  These are tracked and will be resolved before CORE-ACQ split."
  exit 0
else
  echo -e "${GREEN}✓ All boundary checks passed${NC}"
  exit 0
fi
