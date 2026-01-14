// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useAsyncState - Generic async state management hook
 * 
 * Provides a consistent pattern for managing loading, error, and data states
 * during async operations. Reduces boilerplate code and ensures consistent
 * error handling across the application.
 */

import { createSignal, type Accessor, type Setter } from "solid-js";

/** State of an async operation */
export type AsyncStatus = "idle" | "loading" | "success" | "error";

/** Result of an async operation */
export interface AsyncState<T> {
  /** Current data value */
  data: Accessor<T | null>;
  /** Loading status */
  loading: Accessor<boolean>;
  /** Error message if any */
  error: Accessor<string | null>;
  /** Overall status */
  status: Accessor<AsyncStatus>;
  /** Manual data setter (for updates without fetch) */
  setData: Setter<T | null>;
  /** Reset to initial state */
  reset: () => void;
  /** Execute an async function with automatic state management */
  execute: <R = T>(fn: () => Promise<R>, options?: ExecuteOptions<T, R>) => Promise<R | null>;
}

/** Options for execute function */
export interface ExecuteOptions<T, R = T> {
  /** Transform result before storing */
  transform?: (result: R) => T;
  /** Custom error handler */
  onError?: (error: unknown) => string;
  /** Called on success */
  onSuccess?: (result: R) => void;
  /** Don't update data state (useful for side-effect-only operations) */
  skipDataUpdate?: boolean;
  /** Don't reset error before executing */
  keepPreviousError?: boolean;
}

/**
 * Generic async state hook
 * 
 * @param initialData - Initial data value (default: null)
 * @returns AsyncState object with data, loading, error signals and execute function
 * 
 * @example
 * ```tsx
 * const fileState = useAsyncState<FileData>();
 * 
 * // Execute async operation
 * await fileState.execute(
 *   () => invoke<FileData>("read_file", { path }),
 *   { onSuccess: (data) => console.log("Loaded:", data) }
 * );
 * 
 * // In JSX
 * <Show when={fileState.loading()}>Loading...</Show>
 * <Show when={fileState.error()}>{fileState.error()}</Show>
 * <Show when={fileState.data()}>{fileState.data()!.name}</Show>
 * ```
 */
export function useAsyncState<T>(initialData: T | null = null): AsyncState<T> {
  const [data, setData] = createSignal<T | null>(initialData);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  
  const status: Accessor<AsyncStatus> = () => {
    if (loading()) return "loading";
    if (error()) return "error";
    if (data() !== null) return "success";
    return "idle";
  };
  
  const reset = () => {
    setData(() => initialData);
    setLoading(false);
    setError(null);
  };
  
  const execute = async <R = T>(
    fn: () => Promise<R>,
    options?: ExecuteOptions<T, R>
  ): Promise<R | null> => {
    try {
      setLoading(true);
      if (!options?.keepPreviousError) {
        setError(null);
      }
      
      const result = await fn();
      
      if (!options?.skipDataUpdate) {
        const transformedResult = options?.transform 
          ? options.transform(result) 
          : (result as unknown as T);
        setData(() => transformedResult);
      }
      
      options?.onSuccess?.(result);
      return result;
    } catch (e) {
      const errorMessage = options?.onError 
        ? options.onError(e)
        : (e instanceof Error ? e.message : String(e));
      setError(errorMessage);
      return null;
    } finally {
      setLoading(false);
    }
  };
  
  return {
    data,
    loading,
    error,
    status,
    setData,
    reset,
    execute,
  };
}

/**
 * Async state for tracking multiple items by key
 * Useful for loading states keyed by file path, ID, etc.
 */
export interface AsyncSetState<K = string> {
  /** Set of currently loading keys */
  loading: Accessor<Set<K>>;
  /** Check if a specific key is loading */
  isLoading: (key: K) => boolean;
  /** Mark a key as loading */
  startLoading: (key: K) => void;
  /** Mark a key as done loading */
  stopLoading: (key: K) => void;
  /** Execute async operation for a key */
  execute: <T>(key: K, fn: () => Promise<T>) => Promise<T | null>;
  /** Clear all loading states */
  clear: () => void;
}

/**
 * Hook for tracking loading state of multiple items
 * 
 * @example
 * ```tsx
 * const fileLoading = useAsyncSetState<string>();
 * 
 * // Load multiple files
 * await Promise.all(paths.map(path => 
 *   fileLoading.execute(path, () => loadFile(path))
 * ));
 * 
 * // Check specific file loading
 * <Show when={fileLoading.isLoading(path)}>Loading {path}...</Show>
 * ```
 */
export function useAsyncSetState<K = string>(): AsyncSetState<K> {
  const [loading, setLoading] = createSignal<Set<K>>(new Set());
  
  const isLoading = (key: K): boolean => loading().has(key);
  
  const startLoading = (key: K) => {
    setLoading(prev => {
      const next = new Set(prev);
      next.add(key);
      return next;
    });
  };
  
  const stopLoading = (key: K) => {
    setLoading(prev => {
      const next = new Set(prev);
      next.delete(key);
      return next;
    });
  };
  
  const execute = async <T>(key: K, fn: () => Promise<T>): Promise<T | null> => {
    try {
      startLoading(key);
      return await fn();
    } catch (e) {
      console.error(`Error loading ${String(key)}:`, e);
      return null;
    } finally {
      stopLoading(key);
    }
  };
  
  const clear = () => setLoading(new Set<K>());
  
  return {
    loading,
    isLoading,
    startLoading,
    stopLoading,
    execute,
    clear,
  };
}

/**
 * Async state with cache support
 * Caches results by key to avoid redundant fetches
 */
export interface CachedAsyncState<K, V> {
  /** Get cached value for key */
  get: (key: K) => V | undefined;
  /** Check if key has cached value */
  has: (key: K) => boolean;
  /** Check if key is currently loading */
  isLoading: (key: K) => boolean;
  /** Fetch value for key (uses cache if available) */
  fetch: (key: K, fetchFn: () => Promise<V>) => Promise<V | null>;
  /** Force refresh for key (ignores cache) */
  refresh: (key: K, fetchFn: () => Promise<V>) => Promise<V | null>;
  /** Invalidate cache for key */
  invalidate: (key: K) => void;
  /** Clear entire cache */
  clearCache: () => void;
  /** Get all cached entries */
  cache: Accessor<Map<K, V>>;
}

/**
 * Hook for async state with caching
 * 
 * @example
 * ```tsx
 * const containerCache = useCachedAsyncState<string, ContainerInfo>();
 * 
 * // First call fetches, subsequent calls use cache
 * const info = await containerCache.fetch(path, () => invoke("get_info", { path }));
 * 
 * // Force refresh
 * const freshInfo = await containerCache.refresh(path, () => invoke("get_info", { path }));
 * ```
 */
export function useCachedAsyncState<K, V>(): CachedAsyncState<K, V> {
  const [cache, setCache] = createSignal<Map<K, V>>(new Map());
  const [loading, setLoading] = createSignal<Set<K>>(new Set());
  
  const get = (key: K): V | undefined => cache().get(key);
  const has = (key: K): boolean => cache().has(key);
  const isLoading = (key: K): boolean => loading().has(key);
  
  const setLoadingKey = (key: K, value: boolean) => {
    setLoading(prev => {
      const next = new Set(prev);
      if (value) next.add(key);
      else next.delete(key);
      return next;
    });
  };
  
  const setCacheValue = (key: K, value: V) => {
    setCache(prev => {
      const next = new Map(prev);
      next.set(key, value);
      return next;
    });
  };
  
  const fetch = async (key: K, fetchFn: () => Promise<V>): Promise<V | null> => {
    // Return cached value if available
    if (has(key)) {
      return get(key)!;
    }
    
    // Already loading? Wait... but we can't easily wait in this pattern
    // So just start new fetch
    return refresh(key, fetchFn);
  };
  
  const refresh = async (key: K, fetchFn: () => Promise<V>): Promise<V | null> => {
    try {
      setLoadingKey(key, true);
      const result = await fetchFn();
      setCacheValue(key, result);
      return result;
    } catch (e) {
      console.error(`Error fetching ${String(key)}:`, e);
      return null;
    } finally {
      setLoadingKey(key, false);
    }
  };
  
  const invalidate = (key: K) => {
    setCache(prev => {
      const next = new Map(prev);
      next.delete(key);
      return next;
    });
  };
  
  const clearCache = () => {
    setCache(new Map<K, V>());
    setLoading(new Set<K>());
  };
  
  return {
    get,
    has,
    isLoading,
    fetch,
    refresh,
    invalidate,
    clearCache,
    cache,
  };
}
