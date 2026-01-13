// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/* @refresh reload */
import { render } from "solid-js/web";
import { AppRouter } from "./AppRouter";
import "./index.css";

render(() => <AppRouter />, document.getElementById("root") as HTMLElement);
