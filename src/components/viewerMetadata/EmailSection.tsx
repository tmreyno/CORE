// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import type { EmailMetadataSection } from "../../types/viewerMetadata";
import { CollapsibleGroup, MetadataRow } from "./shared";

export function EmailSection(props: { data: EmailMetadataSection }) {
  return (
    <div class="p-3 space-y-3">
      <CollapsibleGroup title="Email Info" defaultOpen>
        <Show when={props.data.subject}>
          <MetadataRow label="Subject" value={props.data.subject!} />
        </Show>
        <Show when={props.data.from}>
          <MetadataRow
            label="From"
            value={props.data.from!}
            highlight
          />
        </Show>
        <Show when={props.data.to && props.data.to.length > 0}>
          <MetadataRow label="To" value={props.data.to!.join(", ")} />
        </Show>
        <Show when={props.data.cc && props.data.cc.length > 0}>
          <MetadataRow label="CC" value={props.data.cc!.join(", ")} />
        </Show>
        <Show when={props.data.date}>
          <MetadataRow
            label="Date"
            value={props.data.date!}
            highlight
          />
        </Show>
      </CollapsibleGroup>

      <CollapsibleGroup title="Technical" defaultOpen={false}>
        <Show when={props.data.messageId}>
          <MetadataRow
            label="Message-ID"
            value={props.data.messageId!}
            mono
            truncate
          />
        </Show>
        <Show when={props.data.inReplyTo}>
          <MetadataRow
            label="In-Reply-To"
            value={props.data.inReplyTo!}
            mono
            truncate
          />
        </Show>
        <Show when={props.data.contentType}>
          <MetadataRow
            label="Content-Type"
            value={props.data.contentType!}
          />
        </Show>
        <Show when={props.data.attachmentCount != null}>
          <MetadataRow
            label="Attachments"
            value={String(props.data.attachmentCount)}
          />
        </Show>
      </CollapsibleGroup>

      <Show when={props.data.messageCount != null}>
        <CollapsibleGroup title="MBOX" defaultOpen>
          <MetadataRow
            label="Messages"
            value={String(props.data.messageCount)}
          />
          <Show when={props.data.selectedMessageIndex != null}>
            <MetadataRow
              label="Viewing"
              value={`Message ${props.data.selectedMessageIndex! + 1}`}
            />
          </Show>
        </CollapsibleGroup>
      </Show>
    </div>
  );
}
