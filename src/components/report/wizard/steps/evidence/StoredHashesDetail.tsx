// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * StoredHashesDetail — shows all stored hashes from container metadata.
 */

import { Show, For } from "solid-js";
import { HiOutlineFingerPrint, HiOutlineShieldCheck } from "../../../../icons";
import type { ContainerInfo } from "../../../../../types/containerInfo";

export function StoredHashesDetail(props: { info: ContainerInfo | undefined }) {
  const storedHashes = () => {
    const ci = props.info;
    if (!ci) return [];

    const hashes: { algorithm: string; hash: string; verified?: boolean | null; source: string }[] = [];

    // EWF stored hashes
    const ewf = ci.e01 || ci.l01;
    if (ewf?.stored_hashes) {
      for (const sh of ewf.stored_hashes) {
        hashes.push({ algorithm: sh.algorithm, hash: sh.hash, verified: sh.verified, source: "EWF" });
      }
    }

    // Companion log hashes
    if (ci.companion_log?.stored_hashes) {
      for (const sh of ci.companion_log.stored_hashes) {
        hashes.push({ algorithm: sh.algorithm, hash: sh.hash, verified: sh.verified, source: "Log" });
      }
    }

    // AD1 companion log hashes
    const ad1Log = ci.ad1?.companion_log;
    if (ad1Log) {
      if (ad1Log.md5_hash) hashes.push({ algorithm: "MD5", hash: ad1Log.md5_hash, source: "AD1 Log" });
      if (ad1Log.sha1_hash) hashes.push({ algorithm: "SHA-1", hash: ad1Log.sha1_hash, source: "AD1 Log" });
      if (ad1Log.sha256_hash) hashes.push({ algorithm: "SHA-256", hash: ad1Log.sha256_hash, source: "AD1 Log" });
    }

    // UFED stored hashes
    if (ci.ufed?.stored_hashes) {
      for (const sh of ci.ufed.stored_hashes) {
        hashes.push({ algorithm: sh.algorithm, hash: sh.hash, source: "UFED" });
      }
    }

    return hashes;
  };

  return (
    <Show when={storedHashes().length > 0}>
      <div class="space-y-1.5 p-2.5 bg-bg/50 rounded-lg border border-border/20">
        <div class="flex items-center gap-1.5 text-xs font-medium text-accent/80 mb-1.5">
          <HiOutlineFingerPrint class="w-3.5 h-3.5" />
          <span>Stored Hashes</span>
          <span class="text-txt/40">({storedHashes().length})</span>
        </div>
        <div class="space-y-1">
          <For each={storedHashes()}>
            {(h) => (
              <div class="flex items-center gap-2 text-xs">
                <span class={`inline-flex items-center gap-1 ${
                  h.verified === true ? 'text-success' :
                  h.verified === false ? 'text-error' : 'text-txt/60'
                }`}>
                  <Show when={h.verified != null}>
                    <HiOutlineShieldCheck class="w-3 h-3" />
                  </Show>
                  <span class="min-w-[50px] text-txt/50">{h.algorithm}</span>
                </span>
                <span class="font-mono text-txt/70 break-all">{h.hash}</span>
                <span class="text-txt/30 text-2xs ml-auto shrink-0">{h.source}</span>
              </div>
            )}
          </For>
        </div>
      </div>
    </Show>
  );
}
