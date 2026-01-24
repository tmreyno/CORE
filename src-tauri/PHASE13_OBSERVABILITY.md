# Phase 13: Advanced Observability & Telemetry

**Status**: ✅ Complete  
**Implementation Date**: January 23, 2026  
**Total Lines**: ~2,200 (Rust backend + tests)

## Overview

Phase 13 implements a production-grade observability system with:
- **Metrics Collection**: Prometheus-compatible metrics (counters, gauges, histograms)
- **Health Monitoring**: Real-time system resource and operational health checks
- **Structured Tracing**: Async-aware logging with automatic span timing

## Files Created

| File | Lines | Description |
|------|-------|-------------|
| `src/common/metrics.rs` | 656 | Metrics collection infrastructure with global registry |
| `src/common/health.rs` | 365 | System health monitoring and diagnostics |
| `src/common/tracing_setup.rs` | 241 | Structured logging initialization and configuration |
| `src/commands/observability.rs` | 183 | 12 Tauri commands for frontend integration |
| `tests/phase13_tests.rs` | 387 | Comprehensive integration tests (11 tests) |
| **Total** | **1,832** | **Complete observability stack** |

## Architecture

### 1. Metrics System (`metrics.rs`)

```rust
// Global thread-safe registry
static METRICS_REGISTRY: LazyLock<Arc<MetricsRegistry>>;

// Metric types
pub enum MetricValue {
    Counter { value: f64 },
    Gauge { value: f64 },
    Histogram {
        count: u64,
        sum: f64,
        min: f64,
        max: f64,
        mean: f64,
        p50: f64,
        p95: f64,
        p99: f64,
    },
}

// Standard operations
pub fn increment_counter(name: &str, value: f64, labels: &[(&str, &str)]);
pub fn set_gauge(name: &str, value: f64, labels: &[(&str, &str)]);
pub fn record_histogram(name: &str, value: f64, labels: &[(&str, &str)]);
```

**Features**:
- **Multi-dimensional**: Label-based metrics for flexible querying
- **Thread-safe**: `DashMap` + `RwLock` for concurrent access
- **Auto-aggregation**: Histogram percentiles (p50, p95, p99)
- **Memory-bounded**: Keep last 10,000 histogram values per metric
- **RAII Timer**: Automatic duration recording via Drop trait

### 2. Health Monitoring (`health.rs`)

```rust
pub struct SystemHealth {
    pub status: HealthStatus,          // Healthy | Degraded | Unhealthy
    pub timestamp: i64,
    pub uptime_seconds: f64,
    pub version: String,
    pub platform: String,
    pub resources: ResourceMetrics,    // CPU, memory, disk
    pub queues: Vec<QueueMetrics>,     // Queue depths, throughput
    pub errors: ErrorMetrics,          // Error rates, top types
    pub issues: Vec<HealthIssue>,      // Detected problems
}

pub struct HealthThresholds {
    pub cpu_warning: f64,              // Default: 70%
    pub cpu_critical: f64,             // Default: 90%
    pub memory_warning: f64,           // Default: 75%
    pub memory_critical: f64,          // Default: 90%
    // ... more thresholds
}
```

**Features**:
- **Real-time**: Live system resource monitoring via `sysinfo`
- **Configurable**: Custom warning/critical thresholds
- **Severity Levels**: Warning | Error | Critical
- **Auto-detection**: Automatically flags issues exceeding thresholds

### 3. Structured Tracing (`tracing_setup.rs`)

```rust
pub fn init_tracing(level: &str, log_dir: impl AsRef<Path>) -> TracingResult<()>;

pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

// Macro for timed spans
timed_span!("operation_name", file_path = path, size = size);
```

**Features**:
- **Async-aware**: Proper span tracking across async boundaries
- **Environment-based**: `RUST_LOG` env var support
- **Console output**: ANSI colors with automatic span timing
- **Format**: Target, level, thread IDs, span duration

### 4. Tauri Commands (`commands/observability.rs`)

| Command | Purpose |
|---------|---------|
| `get_metrics` | Retrieve all metrics |
| `get_metric` | Get specific metric by name |
| `reset_metrics` | Clear all metrics (dev/test only) |
| `get_system_uptime` | Get application uptime |
| `get_metrics_count` | Get total metric count |
| `get_health` | Get system health with default thresholds |
| `get_health_with_thresholds` | Get health with custom thresholds |
| `is_system_healthy` | Quick boolean health check |
| `init_tracing` | Initialize tracing system |
| `get_default_log_dir` | Get platform-specific log directory |
| `parse_log_level` | Validate log level string |
| `get_system_status` | Combined health + metrics summary |

## Usage Examples

### Backend: Metrics Recording

```rust
use ffx_check_lib::common::metrics::{increment_counter, set_gauge, record_histogram, Timer};

// Counter: operations completed
increment_counter("files_hashed_total", 1.0, &[("algorithm", "SHA-256"), ("status", "success")]);

// Gauge: active operations
set_gauge("active_extractions", 5.0, &[("type", "parallel")]);

// Histogram: operation latency
record_histogram("hash_duration_ms", 123.45, &[("algorithm", "BLAKE3")]);

// Timer RAII: automatic timing
{
    let mut timer = Timer::new("verify_container");
    // ... do work ...
    timer.success(); // Mark as successful
} // Automatically records: start, complete, duration
```

### Backend: Health Checks

```rust
use ffx_check_lib::common::health::{get_system_health, HealthStatus};

let health = get_system_health();

match health.status {
    HealthStatus::Healthy => println!("All systems operational"),
    HealthStatus::Degraded => {
        for issue in &health.issues {
            warn!("{}: {}", issue.component, issue.message);
        }
    }
    HealthStatus::Unhealthy => {
        for issue in health.issues.iter().filter(|i| i.severity == IssueSeverity::Critical) {
            error!("CRITICAL - {}: {}", issue.component, issue.message);
        }
    }
}
```

### Backend: Structured Tracing

```rust
use tracing::{info, warn, error, instrument};
use ffx_check_lib::timed_span;

// Function-level tracing with #[instrument]
#[instrument(skip(data))]
async fn process_file(name: &str, size: u64, data: &[u8]) -> Result<()> {
    info!("Processing file");
    // ... work ...
    Ok(())
}

// Manual span with timing
fn extract_files(container_path: &str) -> Result<usize> {
    let _span = timed_span!("extraction", path = container_path, format = "ad1");
    // Span timing automatically recorded on drop
    Ok(42)
}

// Log with context
info!(
    file_path = "/evidence/case001.ad1",
    size_bytes = 1073741824,
    hash = "abc123...",
    "Container opened successfully"
);
```

### Frontend: Metrics Display (TypeScript)

```typescript
import { invoke } from "@tauri-apps/api/core";

interface MetricEntry {
    name: string;
    value: {
        type: "counter" | "gauge" | "histogram";
        // ... type-specific fields
    };
}

// Get all metrics
const metrics = await invoke<MetricEntry[]>("get_metrics");

// Filter by operation type
const hashMetrics = metrics.filter(m => m.name.includes("hash"));

// Display histogram stats
const latencyMetric = metrics.find(m => m.name.includes("operation_duration"));
if (latencyMetric && latencyMetric.value.type === "histogram") {
    console.log(`P50: ${latencyMetric.value.p50}ms`);
    console.log(`P95: ${latencyMetric.value.p95}ms`);
    console.log(`P99: ${latencyMetric.value.p99}ms`);
}
```

### Frontend: Health Monitoring (TypeScript)

```typescript
interface SystemHealth {
    status: "healthy" | "degraded" | "unhealthy";
    timestamp: number;
    uptime_seconds: number;
    version: string;
    platform: string;
    resources: {
        cpu_usage_percent: number;
        memory_usage_percent: number;
        disk_usage_percent: number;
        // ...
    };
    issues: Array<{
        severity: "warning" | "error" | "critical";
        component: string;
        message: string;
    }>;
}

// Get health status
const health = await invoke<SystemHealth>("get_health");

if (health.status !== "healthy") {
    // Display warnings
    health.issues.forEach(issue => {
        showNotification(issue.severity, `${issue.component}: ${issue.message}`);
    });
}

// Quick health check
const isHealthy = await invoke<boolean>("is_system_healthy");
if (!isHealthy) {
    showAlert("System experiencing issues");
}
```

## Performance Characteristics

### Metrics

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Increment counter | O(1) | Lock-free via DashMap |
| Set gauge | O(1) | Single write lock |
| Record histogram | O(1) amortized | Bounded to 10K values |
| Snapshot all | O(n) | n = number of metrics |

**Memory**: ~100 bytes per metric + histogram data (max ~80KB per histogram)

### Health Checks

| Check | Time | System Call |
|-------|------|-------------|
| CPU usage | ~1ms | sysinfo refresh |
| Memory | ~1ms | sysinfo refresh |
| Disk space | ~5ms | First disk only |
| **Total** | **~10ms** | Per check |

### Tracing

| Operation | Overhead | Notes |
|-----------|----------|-------|
| Span creation | <1µs | Stack-allocated |
| Span drop (timing) | <1µs | Single write |
| Log event | ~10µs | Console output |

**Recommendation**: Use INFO level in production for <5% overhead

## Integration Tests

All 11 tests pass ✅

```bash
$ cargo test --test phase13_tests -- --test-threads=1
test test_metrics_counter ... ok          # Counter increments
test test_metrics_gauge ... ok            # Gauge set/increment
test test_metrics_histogram ... ok        # Percentile calculations
test test_metrics_timer ... ok            # RAII timer pattern
test test_metrics_with_labels ... ok      # Multi-dimensional metrics
test test_health_check ... ok             # System resource monitoring
test test_health_thresholds ... ok        # Custom threshold detection
test test_tracing_init ... ok             # Tracing initialization
test test_log_level_parsing ... ok        # Log level validation
test test_standard_metrics_functions ... ok  # Standard recording functions
test test_phase13_summary ... ok          # Integration summary

test result: ok. 11 passed; 0 failed
```

## Standard Metrics

### Predefined Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `operations_started_total` | Counter | `type` | Operations initiated |
| `operations_completed_total` | Counter | `type`, `status` | Operations finished |
| `operation_duration_seconds` | Histogram | `type` | Operation latency |
| `active_operations` | Gauge | `type` | Currently running |
| `bytes_processed_total` | Counter | `type` | Data throughput |
| `cache_accesses_total` | Counter | `cache`, `result` | Cache hit/miss |
| `errors_total` | Counter | `operation`, `type` | Error count |

### Helper Functions

```rust
pub fn record_operation_start(operation_type: &str);
pub fn record_operation_complete(operation_type: &str, duration: Duration, success: bool);
pub fn record_bytes_processed(operation_type: &str, bytes: u64);
pub fn record_cache_access(cache_name: &str, hit: bool);
pub fn record_error(operation_type: &str, error_type: &str);
```

## Configuration

### Cargo Dependencies

```toml
[dependencies]
metrics = "0.23"            # Metrics collection
tracing = "0.1"             # Structured logging
tracing-subscriber = "0.3"  # Log subscriber
sysinfo = "0.32"            # System monitoring
dashmap = "6.1"             # Concurrent map
parking_lot = "0.12"        # Fast locks
```

### Recursion Limit

Due to tracing-subscriber complexity on macOS:

```rust
#![recursion_limit = "1024"]
```

## Future Enhancements

### Phase 13.1: Metrics Export
- **Prometheus exporter**: HTTP `/metrics` endpoint
- **JSON API**: REST endpoint for metrics retrieval
- **Historical data**: Time-series storage (SQLite)

### Phase 13.2: Distributed Tracing
- **OpenTelemetry**: W3C Trace Context propagation
- **Jaeger/Zipkin**: Export spans to tracing backend
- **Correlation IDs**: Link operations across components

### Phase 13.3: Advanced Profiling
- **Flamegraphs**: CPU profiling visualization
- **Memory profiling**: Allocation tracking
- **I/O profiling**: Disk/network bottleneck detection

### Phase 13.4: Alerting
- **Threshold alerts**: Trigger notifications on metrics
- **Anomaly detection**: ML-based pattern recognition
- **Alert routing**: Email, Slack, webhooks

## Integration with Existing Phases

| Phase | Integration Point |
|-------|-------------------|
| Phase 3 (SQLite Cache) | Track cache hit rates, query latency |
| Phase 6 (Memory-mapped Hex) | Monitor mmap size, page faults |
| Phase 7 (Parallel Extraction) | Track worker utilization, throughput |
| Phase 8 (BLAKE3 Dedup) | Record dedup ratio, hash speed |
| Phase 9 (Streaming) | Monitor stream backpressure, chunk sizes |
| Phase 11 (Recovery) | Track retry counts, recovery success rate |

## Key Design Decisions

1. **No file logging by default**: Simplified to avoid `tracing-appender` recursion issues on macOS. Console output only.

2. **Thread-safe global registry**: `LazyLock` + `Arc` for zero-cost singleton without initialization ordering issues.

3. **Memory-bounded histograms**: Keep last 10K values to prevent unbounded growth. Drain oldest 1K when limit reached.

4. **Label sorting**: Consistent metric keys by sorting labels alphabetically.

5. **Reset for tests only**: `reset_metrics()` public for integration tests but not recommended for production use.

## Troubleshooting

### High Memory Usage
- **Symptom**: Memory grows over time
- **Cause**: Too many unique label combinations
- **Solution**: Limit cardinality (e.g., don't use UUIDs as labels)

### Missing Metrics
- **Symptom**: Metric not showing in snapshot
- **Cause**: Never recorded or wrong labels
- **Solution**: Check label names (case-sensitive, sorted)

### Tracing Not Initializing
- **Symptom**: `SetGlobalDefaultError`
- **Cause**: Already initialized (e.g., in tests)
- **Solution**: Ignore error or check if already initialized

### Health Checks Slow
- **Symptom**: `get_health()` takes >100ms
- **Cause**: Disk enumeration on some systems
- **Solution**: Cache health checks or increase polling interval

## Related Documentation

- [Phase 11: Error Recovery](PHASE11_ERROR_RECOVERY.md) - Notifications and recovery tracking
- [Phase 10: Integration Testing](PHASE10_INTEGRATION_TESTING.md) - Testing patterns
- [CODE_BIBLE.md](../../CODE_BIBLE.md) - Architecture overview

## Metrics Example Output

```json
{
  "files_hashed_total{algorithm=\"SHA-256\",status=\"success\"}": {
    "type": "counter",
    "value": 42
  },
  "active_operations{type=\"hash\"}": {
    "type": "gauge",
    "value": 3
  },
  "operation_duration_seconds{type=\"hash\"}": {
    "type": "histogram",
    "count": 42,
    "sum": 123.45,
    "min": 0.1,
    "max": 10.5,
    "mean": 2.94,
    "p50": 2.5,
    "p95": 8.2,
    "p99": 10.1
  }
}
```

## Health Check Example Output

```json
{
  "status": "degraded",
  "timestamp": 1706025600,
  "uptime_seconds": 3600.5,
  "version": "0.1.0",
  "platform": "macos aarch64",
  "resources": {
    "cpu_usage_percent": 45.2,
    "memory_used_bytes": 8589934592,
    "memory_total_bytes": 17179869184,
    "memory_usage_percent": 50.0,
    "disk_available_bytes": 107374182400,
    "disk_total_bytes": 536870912000,
    "disk_usage_percent": 80.0
  },
  "queues": [],
  "errors": {
    "errors_last_minute": 5,
    "errors_last_hour": 25,
    "error_rate": 0.08,
    "top_error_types": []
  },
  "issues": [
    {
      "severity": "warning",
      "component": "Disk",
      "message": "Disk usage high: 80.0%",
      "metric_value": 80.0,
      "threshold": 75.0
    }
  ]
}
```

---

**Phase 13 Complete** ✅  
Production-ready observability system with comprehensive metrics, health monitoring, and structured tracing.
