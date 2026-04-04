#!/bin/bash
# =============================================================================
# CORE-FFX Container Testing Script
# Copyright (c) 2024-2026 CORE-FFX Project Contributors
# Licensed under MIT License - see LICENSE file for details
# =============================================================================

# Quick automated test to verify all container types can be discovered

set -e

CASE_DIR=""
for candidate in \
    "/Users/terryreynolds/Cases/1827-1001" \
    "/Users/terryreynolds/1827-1001 Case With Data "
do
    if [ -d "$candidate/1.Evidence" ]; then
        CASE_DIR="$candidate"
        break
    fi
done

EVIDENCE_DIR="$CASE_DIR/1.Evidence"

echo "======================================"
echo "CORE-FFX Container Discovery Test"
echo "======================================"
echo ""

if [ -z "$CASE_DIR" ] || [ ! -d "$EVIDENCE_DIR" ]; then
    echo "❌ Evidence directory not found in known case locations"
    echo "   Checked:"
    echo "   - /Users/terryreynolds/Cases/1827-1001/1.Evidence"
    echo "   - /Users/terryreynolds/1827-1001 Case With Data /1.Evidence"
    exit 1
fi

echo "📁 Evidence Directory: $EVIDENCE_DIR"
echo ""

# Count container types
echo "🔍 Scanning for forensic containers..."
echo ""

AD1_FILES=$(find "$EVIDENCE_DIR" -type f -iname "*.ad1" 2>/dev/null | wc -l | xargs)
E01_FILES=$(find "$EVIDENCE_DIR" -type f -iname "*.e01" 2>/dev/null | wc -l | xargs)
L01_FILES=$(find "$EVIDENCE_DIR" -type f -iname "*.l01" 2>/dev/null | wc -l | xargs)
ZIP_FILES=$(find "$EVIDENCE_DIR" -type f -iname "*.zip" 2>/dev/null | wc -l | xargs)
SEVENZ_FILES=$(find "$EVIDENCE_DIR" -type f -iname "*.7z" 2>/dev/null | wc -l | xargs)
DMG_FILES=$(find "$EVIDENCE_DIR" -type f -iname "*.dmg" 2>/dev/null | wc -l | xargs)
UFD_FILES=$(find "$EVIDENCE_DIR" -type f \( -iname "*.ufd" -o -iname "*.ufdr" -o -iname "*.ufdx" \) 2>/dev/null | wc -l | xargs)
ISO_FILES=$(find "$EVIDENCE_DIR" -type f -iname "*.iso" 2>/dev/null | wc -l | xargs)
RAW_FILES=$(find "$EVIDENCE_DIR" -type f \( -iname "*.raw" -o -iname "*.dd" -o -iname "*.img" \) 2>/dev/null | wc -l | xargs)

echo "Container Type Summary:"
echo "----------------------"
echo "  AD1 (AccessData):     $AD1_FILES files"
echo "  E01 (EnCase):         $E01_FILES files"
echo "  L01 (Logical):        $L01_FILES files"
echo "  ZIP Archives:         $ZIP_FILES files"
echo "  7z Archives:          $SEVENZ_FILES files"
echo "  DMG (Apple):          $DMG_FILES files"
echo "  UFED (Mobile):        $UFD_FILES files"
echo "  ISO Images:           $ISO_FILES files"
echo "  RAW Images:           $RAW_FILES files"
echo ""

TOTAL=$((AD1_FILES + E01_FILES + L01_FILES + ZIP_FILES + SEVENZ_FILES + DMG_FILES + UFD_FILES + ISO_FILES + RAW_FILES))
echo "📊 Total Containers: $TOTAL"
echo ""

# List first 10 containers found
echo "📋 Sample Container Files (first 10):"
echo "------------------------------------"
find "$EVIDENCE_DIR" -type f \( \
    -iname "*.ad1" -o \
    -iname "*.e01" -o \
    -iname "*.l01" -o \
    -iname "*.zip" -o \
    -iname "*.7z" -o \
    -iname "*.dmg" -o \
    -iname "*.ufd*" -o \
    -iname "*.iso" -o \
    -iname "*.raw" -o \
    -iname "*.dd" -o \
    -iname "*.img" \
\) 2>/dev/null | head -10 | while read -r file; do
    SIZE=$(ls -lh "$file" | awk '{print $5}')
    BASENAME=$(basename "$file")
    EXT="${BASENAME##*.}"
    echo "  • $BASENAME ($SIZE) [$EXT]"
done

echo ""

# Check if app is running
echo "🚀 Application Status:"
echo "---------------------"
if pgrep -f "npm run tauri dev" > /dev/null || pgrep -f "ffx" > /dev/null; then
    echo "  ✅ CORE-FFX appears to be running"
    echo "  💡 Open the app and load the case from: $CASE_DIR"
else
    echo "  ⚠️  CORE-FFX not detected"
    echo "  💡 Run: cd ~/GitHub/CORE-1 && npm run tauri dev"
fi

echo ""

# Test instructions
echo "📝 Manual Test Steps:"
echo "---------------------"
echo "1. Open CORE-FFX application"
echo "2. Open or create a project using evidence from: $EVIDENCE_DIR"
echo "3. Navigate to Evidence Tree (left panel)"
echo "4. Scan the evidence directory if containers are not visible"
echo "5. For each container type:"
echo "   a. Click chevron to expand"
echo "   b. Verify tree loads correctly"
echo "   c. Navigate folders/files"
echo "   d. Select a file and view content"
echo "   e. Check performance (should be < 5 sec for most)"
echo "6. Specifically confirm:"
echo "   a. L01 and UFED use lazy logical trees and open files inline"
echo "   b. DMG and ISO browse through the archive-style tree"
echo "   c. Nested containers expand inline inside AD1, VFS, archives, and lazy trees"
echo ""

# Check for specific test files
echo "🎯 Recommended Test Files:"
echo "-------------------------"
if [ $E01_FILES -gt 0 ]; then
    E01_FILE=$(find "$EVIDENCE_DIR" -type f -iname "*.e01" 2>/dev/null | head -1)
    echo "  E01: $(basename "$E01_FILE")"
    echo "    → Test VFS mounting and partition detection"
fi

if [ $L01_FILES -gt 0 ]; then
    L01_FILE=$(find "$EVIDENCE_DIR" -type f -iname "*.l01" 2>/dev/null | head -1)
    echo "  L01: $(basename "$L01_FILE")"
    echo "    → Test lazy logical tree browsing and inline viewers"
fi

if [ $UFD_FILES -gt 0 ]; then
    UFD_FILE=$(find "$EVIDENCE_DIR" -type f \( -iname "*.ufd" -o -iname "*.ufdr" -o -iname "*.ufdx" \) 2>/dev/null | head -1)
    echo "  UFED: $(basename "$UFD_FILE")"
    echo "    → Test lazy mobile extraction tree browsing and inline viewers"
fi

if [ $DMG_FILES -gt 0 ]; then
    DMG_FILE=$(find "$EVIDENCE_DIR" -type f -iname "*.dmg" 2>/dev/null | head -1)
    echo "  DMG: $(basename "$DMG_FILE")"
    echo "    → Test archive-style tree browsing and inline file preview"
fi

if [ $ISO_FILES -gt 0 ]; then
    ISO_FILE=$(find "$EVIDENCE_DIR" -type f -iname "*.iso" 2>/dev/null | head -1)
    echo "  ISO: $(basename "$ISO_FILE")"
    echo "    → Test archive-style tree browsing and inline file preview"
fi

if [ $ZIP_FILES -gt 0 ]; then
    ZIP_FILE=$(find "$EVIDENCE_DIR" -type f -iname "*.zip" 2>/dev/null | head -1)
    echo "  ZIP: $(basename "$ZIP_FILE")"
    echo "    → Test archive tree loading and nested container expansion"
fi

echo ""
echo "✅ Container scan complete!"
echo ""
