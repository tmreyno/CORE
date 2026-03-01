#!/usr/bin/env bash
# =============================================================================
# CORE-FFX - Forensic File Explorer
# Copyright (c) 2024-2026 CORE-FFX Project Contributors
# Licensed under MIT License - see LICENSE file for details
# =============================================================================
#
# Build Windows executables from macOS
#
# Usage:
#   ./scripts/build-windows.sh              # Default: local cross-compile (.exe only)
#   ./scripts/build-windows.sh --local      # Local cross-compile via cargo-xwin
#   ./scripts/build-windows.sh --ci         # Trigger full CI build (MSI + NSIS + EXE)
#   ./scripts/build-windows.sh --ci --wait  # Trigger CI and wait for completion
#   ./scripts/build-windows.sh --docker     # Cross-compile in Docker container
#   ./scripts/build-windows.sh --setup      # Install cross-compilation toolchain
#   ./scripts/build-windows.sh --check      # Verify toolchain is ready
#   ./scripts/build-windows.sh --prebuild   # Trigger pre-build of native deps in CI
#
# Modes:
#   --local   Fast local .exe (C FFI libs use stubs — 7z/EWF creation disabled)
#   --ci      Full Windows build with MSI/NSIS installers via GitHub Actions
#   --docker  Full cross-compile in Docker (requires Docker Desktop)
#
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TAURI_DIR="$PROJECT_ROOT/src-tauri"
TARGET="x86_64-pc-windows-msvc"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color
BOLD='\033[1m'

info()    { echo -e "${BLUE}ℹ${NC}  $*"; }
success() { echo -e "${GREEN}✅${NC} $*"; }
warn()    { echo -e "${YELLOW}⚠️${NC}  $*"; }
error()   { echo -e "${RED}❌${NC} $*" >&2; }
header()  { echo -e "\n${BOLD}${CYAN}═══ $* ═══${NC}\n"; }

# ─── Setup: Install cross-compilation toolchain ─────────────────────────────

cmd_setup() {
    header "Setting up Windows cross-compilation toolchain"

    # 1. Rust target
    info "Adding Rust target: $TARGET"
    rustup target add "$TARGET"
    success "Rust target $TARGET added"

    # 2. cargo-xwin
    if command -v cargo-xwin &>/dev/null; then
        success "cargo-xwin already installed"
    else
        info "Installing cargo-xwin (downloads Windows SDK + CRT automatically)..."
        cargo install cargo-xwin
        success "cargo-xwin installed"
    fi

    # 3. Verify clang/llvm for C compilation
    if command -v clang &>/dev/null; then
        success "clang found: $(clang --version | head -1)"
    else
        warn "clang not found — installing via Xcode Command Line Tools..."
        xcode-select --install 2>/dev/null || true
    fi

    # 4. Check for Docker (optional)
    if command -v docker &>/dev/null; then
        success "Docker found: $(docker --version)"
    else
        info "Docker not found (optional — needed for --docker mode)"
    fi

    # 5. Check for gh CLI (optional)
    if command -v gh &>/dev/null; then
        success "GitHub CLI found: $(gh --version | head -1)"
    else
        info "GitHub CLI not found (optional — needed for --ci mode)"
        info "Install: brew install gh"
    fi

    echo ""
    success "Setup complete! Run: ./scripts/build-windows.sh --check"
}

# ─── Check: Verify toolchain readiness ──────────────────────────────────────

cmd_check() {
    header "Checking Windows cross-compilation toolchain"

    local ok=true

    # Rust
    if rustup target list --installed | grep -q "$TARGET"; then
        success "Rust target: $TARGET"
    else
        error "Rust target $TARGET not installed"
        ok=false
    fi

    # cargo-xwin
    if command -v cargo-xwin &>/dev/null; then
        success "cargo-xwin: $(cargo xwin --version 2>/dev/null || echo 'installed')"
    else
        error "cargo-xwin not installed"
        ok=false
    fi

    # clang
    if command -v clang &>/dev/null; then
        success "clang: $(clang --version 2>/dev/null | head -1)"
    else
        error "clang not found"
        ok=false
    fi

    # Node.js
    if command -v node &>/dev/null; then
        success "Node.js: $(node --version)"
    else
        error "Node.js not found"
        ok=false
    fi

    # npm dependencies
    if [ -d "$PROJECT_ROOT/node_modules" ]; then
        success "npm dependencies installed"
    else
        warn "npm dependencies not installed (run: npm install)"
    fi

    # Pre-built Windows native libs
    echo ""
    info "Pre-built Windows native libraries:"
    local prebuilt_archive="$PROJECT_ROOT/patches/libarchive2-sys/prebuilt/windows-x64-msvc/archive.lib"
    local prebuilt_7z="$PROJECT_ROOT/sevenzip-ffi/build/Release/7z_ffi.lib"
    local prebuilt_ewf="$PROJECT_ROOT/libewf-ffi/prebuilt/windows-x64-msvc/ewf.lib"

    if [ -f "$prebuilt_archive" ]; then
        success "  archive.lib ($(du -h "$prebuilt_archive" | cut -f1))"
    else
        warn "  archive.lib — MISSING (will use CMake fallback or fail)"
    fi

    if [ -f "$prebuilt_7z" ]; then
        success "  7z_ffi.lib ($(du -h "$prebuilt_7z" | cut -f1))"
    else
        warn "  7z_ffi.lib — MISSING (stub fallback: 7z features disabled)"
    fi

    if [ -f "$prebuilt_ewf" ]; then
        success "  ewf.lib ($(du -h "$prebuilt_ewf" | cut -f1))"
    else
        warn "  ewf.lib — MISSING (stub fallback: libewf features disabled)"
    fi

    # Docker
    echo ""
    if command -v docker &>/dev/null; then
        success "Docker: $(docker --version)"
    else
        info "Docker: not installed (optional, for --docker mode)"
    fi

    # gh CLI
    if command -v gh &>/dev/null; then
        if gh auth status &>/dev/null 2>&1; then
            success "GitHub CLI: authenticated"
        else
            warn "GitHub CLI: installed but not authenticated (run: gh auth login)"
        fi
    else
        info "GitHub CLI: not installed (optional, for --ci mode)"
    fi

    echo ""
    if $ok; then
        success "Toolchain ready for local cross-compilation!"
    else
        error "Missing dependencies — run: ./scripts/build-windows.sh --setup"
    fi
}

# ─── Local Build: cargo-xwin cross-compilation ──────────────────────────────

cmd_local() {
    header "Cross-compiling CORE-FFX for Windows (local)"

    # Verify toolchain
    if ! rustup target list --installed | grep -q "$TARGET"; then
        error "Rust target $TARGET not installed. Run: ./scripts/build-windows.sh --setup"
        exit 1
    fi
    if ! command -v cargo-xwin &>/dev/null; then
        error "cargo-xwin not installed. Run: ./scripts/build-windows.sh --setup"
        exit 1
    fi

    # Build frontend first
    info "Building frontend (SolidJS + Vite)..."
    cd "$PROJECT_ROOT"
    npm run build
    success "Frontend built → dist/"

    # Cross-compile Rust backend
    info "Cross-compiling Rust backend for $TARGET..."
    info "Note: C FFI libraries (sevenzip-ffi, libewf-ffi) will use stub fallbacks"
    info "      unless pre-built .lib files are present."
    echo ""

    cd "$TAURI_DIR"

    # Set up environment for cross-compilation
    # cargo-xwin handles Windows SDK + MSVC CRT automatically
    export CARGO_BUILD_TARGET="$TARGET"

    # Disable sccache — it intercepts rustc and passes host-native flags
    # (like -C target-cpu=native) that break cross-compilation to Windows.
    unset RUSTC_WRAPPER

    # Override .cargo/config.toml settings that break cross-compilation:
    # - target-cpu=native would emit ARM instructions for a Windows x86_64 build
    # - rustc-wrapper=sccache conflicts with cargo-xwin's own rustc wrapping
    export CARGO_BUILD_RUSTFLAGS="-C target-cpu=x86-64"
    export CARGO_BUILD_RUSTC_WRAPPER=""

    # Build with cargo-xwin
    # Use --no-default-features + explicit features to exclude 'unrar'
    # (unrar's C++ vendor code uses x86 SSE/SSSE3 intrinsics that fail
    #  when cross-compiling from ARM; RAR reading still works via libarchive)
    cargo xwin build \
        --release \
        --target "$TARGET" \
        --no-default-features --features ai-assistant \
        2>&1 | while IFS= read -r line; do
        # Filter verbose output, show key milestones
        if [[ "$line" == *"Compiling core-ffx"* ]]; then
            info "$line"
        elif [[ "$line" == *"Compiling tauri"* ]]; then
            info "$line"
        elif [[ "$line" == *"warning"*"stub"* ]]; then
            warn "$line"
        elif [[ "$line" == *"error"* ]]; then
            error "$line"
        elif [[ "$line" == *"Finished"* ]]; then
            success "$line"
        fi
    done

    # Check output
    local exe_path="$TAURI_DIR/target/$TARGET/release/core-ffx.exe"
    if [ -f "$exe_path" ]; then
        local size
        size=$(du -h "$exe_path" | cut -f1)
        echo ""
        header "Build Complete"
        success "Windows executable: $exe_path"
        success "Size: $size"
        echo ""
        info "To test on Windows, copy the .exe and the dist/ folder."
        info "Note: WebView2 runtime must be installed on the Windows machine."
        echo ""
        warn "Stub limitations (unless pre-built .lib files are present):"
        warn "  • 7z archive creation: returns error at runtime"
        warn "  • EWF image creation: returns error at runtime"
        warn "  • All other features (parsing, viewing, hashing, etc.) work normally"
        echo ""
        info "For a full build with MSI/NSIS installers, use: ./scripts/build-windows.sh --ci"
    else
        error "Build failed — executable not found at: $exe_path"
        info "Check the cargo output above for errors."
        exit 1
    fi
}

# ─── CI Build: Trigger GitHub Actions ────────────────────────────────────────

cmd_ci() {
    local wait=false
    if [[ "${1:-}" == "--wait" ]]; then
        wait=true
    fi

    header "Triggering Windows build via GitHub Actions"

    # Verify gh CLI
    if ! command -v gh &>/dev/null; then
        error "GitHub CLI (gh) required. Install: brew install gh"
        exit 1
    fi
    if ! gh auth status &>/dev/null 2>&1; then
        error "GitHub CLI not authenticated. Run: gh auth login"
        exit 1
    fi

    # Determine version from Cargo.toml
    local version
    version=$(grep -m1 '^version' "$TAURI_DIR/Cargo.toml" | sed 's/.*"\(.*\)"/\1/')
    local tag="v${version}"

    info "Version: $version (tag: $tag)"
    info "Repository: tmreyno/CORE"
    echo ""

    # Ask user which mode
    echo -e "${BOLD}Choose build trigger:${NC}"
    echo "  1) Dispatch release workflow (creates draft release with MSI + EXE)"
    echo "  2) Dispatch release for existing tag"
    echo "  3) Just build pre-built native deps (prebuild workflow)"
    echo ""
    read -rp "Choice [1]: " choice
    choice="${choice:-1}"

    case "$choice" in
        1)
            info "Dispatching release workflow with tag $tag..."
            gh workflow run release.yml \
                --repo tmreyno/CORE \
                -f tag_name="$tag"
            success "Release workflow dispatched!"
            ;;
        2)
            read -rp "Enter tag name (e.g., v0.1.11): " custom_tag
            info "Dispatching release workflow for $custom_tag..."
            gh workflow run release.yml \
                --repo tmreyno/CORE \
                -f tag_name="$custom_tag"
            success "Release workflow dispatched for $custom_tag!"
            ;;
        3)
            info "Dispatching pre-build native deps workflow..."
            gh workflow run prebuild-native-deps.yml \
                --repo tmreyno/CORE \
                -f build_libarchive=true \
                -f build_sevenzip=true \
                -f build_libewf=true \
                -f create_pr=true
            success "Pre-build workflow dispatched!"
            ;;
        *)
            error "Invalid choice"
            exit 1
            ;;
    esac

    echo ""
    info "Monitor at: https://github.com/tmreyno/CORE/actions"

    if $wait; then
        info "Waiting for workflow to complete..."
        echo ""
        # Wait for the run to appear
        sleep 5

        # Get the latest run ID
        local run_id
        run_id=$(gh run list --repo tmreyno/CORE --workflow release.yml --limit 1 --json databaseId --jq '.[0].databaseId' 2>/dev/null || echo "")

        if [ -n "$run_id" ]; then
            info "Watching run #$run_id..."
            gh run watch "$run_id" --repo tmreyno/CORE --exit-status || true

            # Download artifacts if successful
            local status
            status=$(gh run view "$run_id" --repo tmreyno/CORE --json conclusion --jq '.conclusion')
            if [ "$status" = "success" ]; then
                echo ""
                success "Build completed successfully!"
                read -rp "Download Windows artifacts? [Y/n]: " dl
                dl="${dl:-Y}"
                if [[ "$dl" =~ ^[Yy] ]]; then
                    local artifact_dir="$PROJECT_ROOT/build-artifacts/windows"
                    mkdir -p "$artifact_dir"
                    info "Downloading to $artifact_dir..."
                    gh run download "$run_id" \
                        --repo tmreyno/CORE \
                        --dir "$artifact_dir" \
                        --pattern "*windows*" 2>/dev/null || \
                    gh release download "$tag" \
                        --repo tmreyno/CORE \
                        --dir "$artifact_dir" \
                        --pattern "*.msi" \
                        --pattern "*.exe" \
                        --clobber 2>/dev/null || true
                    success "Artifacts downloaded to: $artifact_dir"
                    ls -lh "$artifact_dir"/*  2>/dev/null || true
                fi
            else
                warn "Build status: $status"
                info "Check logs: gh run view $run_id --repo tmreyno/CORE --log"
            fi
        else
            warn "Could not find run ID. Monitor manually at:"
            info "https://github.com/tmreyno/CORE/actions"
        fi
    fi
}

# ─── Docker Build: Full cross-compilation in container ───────────────────────

cmd_docker() {
    header "Cross-compiling CORE-FFX for Windows (Docker)"

    if ! command -v docker &>/dev/null; then
        error "Docker not found. Install Docker Desktop: https://docker.com/products/docker-desktop"
        exit 1
    fi

    local dockerfile="$PROJECT_ROOT/scripts/Dockerfile.windows-cross"
    if [ ! -f "$dockerfile" ]; then
        error "Dockerfile not found: $dockerfile"
        exit 1
    fi

    # Build the Docker image
    info "Building cross-compilation Docker image (first run downloads ~2GB)..."
    docker build \
        -t core-ffx-windows-cross \
        -f "$dockerfile" \
        "$PROJECT_ROOT"

    # Run the build
    info "Running Windows cross-compilation in container..."
    local output_dir="$PROJECT_ROOT/build-artifacts/windows-docker"
    mkdir -p "$output_dir"

    docker run --rm \
        -v "$PROJECT_ROOT:/workspace:ro" \
        -v "$output_dir:/output" \
        core-ffx-windows-cross

    # Check output
    if ls "$output_dir"/*.exe &>/dev/null 2>&1; then
        header "Docker Build Complete"
        success "Artifacts:"
        ls -lh "$output_dir"/*
    else
        error "Docker build did not produce expected output"
        exit 1
    fi
}

# ─── Prebuild: Trigger native deps prebuild ─────────────────────────────────

cmd_prebuild() {
    header "Triggering Windows native dependency pre-build"

    if ! command -v gh &>/dev/null; then
        error "GitHub CLI (gh) required. Install: brew install gh"
        exit 1
    fi

    info "Dispatching prebuild-native-deps workflow..."
    gh workflow run prebuild-native-deps.yml \
        --repo tmreyno/CORE \
        -f build_libarchive=true \
        -f build_sevenzip=true \
        -f build_libewf=true \
        -f create_pr=true

    success "Pre-build workflow dispatched!"
    info "Monitor at: https://github.com/tmreyno/CORE/actions/workflows/prebuild-native-deps.yml"
    echo ""
    info "After completion, merge the PR to get pre-built .lib files in the repo."
    info "This speeds up future --local builds and CI release builds."
}

# ─── Main ────────────────────────────────────────────────────────────────────

usage() {
    echo -e "${BOLD}CORE-FFX Windows Build Tool${NC}"
    echo ""
    echo "Build Windows executables from macOS."
    echo ""
    echo -e "${BOLD}Usage:${NC}"
    echo "  $(basename "$0") [mode] [options]"
    echo ""
    echo -e "${BOLD}Modes:${NC}"
    echo "  --local      Cross-compile locally via cargo-xwin (default)"
    echo "               Produces .exe — fast, but C FFI features use stubs"
    echo "  --ci         Trigger full CI build via GitHub Actions"
    echo "               Produces MSI + NSIS installer + .exe (full features)"
    echo "  --docker     Cross-compile in Docker container"
    echo "               Full build without Windows machine"
    echo ""
    echo -e "${BOLD}Setup:${NC}"
    echo "  --setup      Install cross-compilation toolchain"
    echo "  --check      Verify toolchain is ready"
    echo "  --prebuild   Trigger native dependency pre-build in CI"
    echo ""
    echo -e "${BOLD}Options:${NC}"
    echo "  --wait       (with --ci) Wait for CI completion and download artifacts"
    echo ""
    echo -e "${BOLD}Examples:${NC}"
    echo "  $(basename "$0") --setup          # First-time setup"
    echo "  $(basename "$0")                  # Quick local .exe build"
    echo "  $(basename "$0") --ci --wait      # Full CI build, wait & download"
    echo "  $(basename "$0") --docker         # Docker-based cross-compile"
    echo ""
    echo -e "${BOLD}Quick Start:${NC}"
    echo "  1. ./scripts/build-windows.sh --setup"
    echo "  2. ./scripts/build-windows.sh --check"
    echo "  3. ./scripts/build-windows.sh"
    echo ""
}

main() {
    local mode="${1:---local}"
    shift 2>/dev/null || true

    case "$mode" in
        --local|-l)     cmd_local ;;
        --ci|-c)        cmd_ci "$@" ;;
        --docker|-d)    cmd_docker ;;
        --setup|-s)     cmd_setup ;;
        --check)        cmd_check ;;
        --prebuild|-p)  cmd_prebuild ;;
        --help|-h)      usage ;;
        *)
            error "Unknown mode: $mode"
            usage
            exit 1
            ;;
    esac
}

main "$@"
