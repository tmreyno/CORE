// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Shared helpers for best-effort frontend event delivery.

pub(crate) fn log_emit_result(event: &str, result: tauri::Result<()>) -> bool {
    match result {
        Ok(()) => true,
        Err(error) => {
            tracing::debug!(event, %error, "Frontend event delivery failed");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_emit_result_returns_true_for_success() {
        assert!(log_emit_result("test-event", Ok(())));
    }
}
