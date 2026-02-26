// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import type { ExifMetadataSection } from "../../types/viewerMetadata";
import { CollapsibleGroup, MetadataRow } from "./shared";

export function ExifSection(props: { data: ExifMetadataSection }) {
  return (
    <div class="p-3 space-y-3">
      {/* Camera */}
      <Show when={props.data.make || props.data.model}>
        <CollapsibleGroup title="Camera" defaultOpen>
          <Show when={props.data.make}>
            <MetadataRow label="Make" value={props.data.make!} />
          </Show>
          <Show when={props.data.model}>
            <MetadataRow label="Model" value={props.data.model!} />
          </Show>
          <Show when={props.data.lensModel}>
            <MetadataRow label="Lens" value={props.data.lensModel!} />
          </Show>
          <Show when={props.data.software}>
            <MetadataRow label="Software" value={props.data.software!} />
          </Show>
        </CollapsibleGroup>
      </Show>

      {/* Capture Settings */}
      <Show
        when={
          props.data.exposureTime || props.data.fNumber || props.data.iso
        }
      >
        <CollapsibleGroup title="Capture Settings" defaultOpen>
          <Show when={props.data.exposureTime}>
            <MetadataRow
              label="Exposure"
              value={props.data.exposureTime!}
            />
          </Show>
          <Show when={props.data.fNumber}>
            <MetadataRow
              label="Aperture"
              value={"f/" + props.data.fNumber}
            />
          </Show>
          <Show when={props.data.iso}>
            <MetadataRow label="ISO" value={String(props.data.iso)} />
          </Show>
          <Show when={props.data.focalLength}>
            <MetadataRow
              label="Focal Length"
              value={props.data.focalLength!}
            />
          </Show>
          <Show when={props.data.flash}>
            <MetadataRow label="Flash" value={props.data.flash!} />
          </Show>
        </CollapsibleGroup>
      </Show>

      {/* Timestamps (forensically critical) */}
      <Show when={props.data.dateTimeOriginal || props.data.dateTime}>
        <CollapsibleGroup title="Timestamps" defaultOpen>
          <Show when={props.data.dateTimeOriginal}>
            <MetadataRow
              label="Original"
              value={props.data.dateTimeOriginal!}
              highlight
            />
          </Show>
          <Show when={props.data.dateTimeDigitized}>
            <MetadataRow
              label="Digitized"
              value={props.data.dateTimeDigitized!}
            />
          </Show>
          <Show when={props.data.dateTime}>
            <MetadataRow
              label="Modified"
              value={props.data.dateTime!}
            />
          </Show>
          <Show when={props.data.gpsTimestamp}>
            <MetadataRow
              label="GPS Time"
              value={props.data.gpsTimestamp!}
              highlight
            />
          </Show>
        </CollapsibleGroup>
      </Show>

      {/* GPS */}
      <Show when={props.data.gps}>
        <CollapsibleGroup title="GPS Location" defaultOpen>
          <MetadataRow
            label="Latitude"
            value={`${props.data.gps!.latitude.toFixed(6)}° ${props.data.gps!.latitudeRef}`}
            highlight
          />
          <MetadataRow
            label="Longitude"
            value={`${props.data.gps!.longitude.toFixed(6)}° ${props.data.gps!.longitudeRef}`}
            highlight
          />
          <Show when={props.data.gps!.altitude != null}>
            <MetadataRow
              label="Altitude"
              value={`${props.data.gps!.altitude!.toFixed(1)}m`}
            />
          </Show>
          <a
            href={`https://www.google.com/maps?q=${props.data.gps!.latitude},${props.data.gps!.longitude}`}
            target="_blank"
            rel="noopener noreferrer"
            class="block mt-1 text-xs text-accent hover:text-accent-hover underline"
          >
            Open in Google Maps ↗
          </a>
        </CollapsibleGroup>
      </Show>

      {/* Image Dimensions */}
      <Show when={props.data.width || props.data.height}>
        <CollapsibleGroup title="Image" defaultOpen={false}>
          <Show when={props.data.width && props.data.height}>
            <MetadataRow
              label="Dimensions"
              value={`${props.data.width} × ${props.data.height}`}
            />
          </Show>
          <Show when={props.data.orientation}>
            <MetadataRow
              label="Orientation"
              value={String(props.data.orientation)}
            />
          </Show>
          <Show when={props.data.colorSpace}>
            <MetadataRow
              label="Color Space"
              value={props.data.colorSpace!}
            />
          </Show>
        </CollapsibleGroup>
      </Show>

      {/* Forensic Identifiers */}
      <Show
        when={
          props.data.imageUniqueId ||
          props.data.serialNumber ||
          props.data.ownerName
        }
      >
        <CollapsibleGroup title="Forensic" defaultOpen>
          <Show when={props.data.imageUniqueId}>
            <MetadataRow
              label="Unique ID"
              value={props.data.imageUniqueId!}
              highlight
              mono
            />
          </Show>
          <Show when={props.data.serialNumber}>
            <MetadataRow
              label="Serial #"
              value={props.data.serialNumber!}
              highlight
              mono
            />
          </Show>
          <Show when={props.data.ownerName}>
            <MetadataRow label="Owner" value={props.data.ownerName!} />
          </Show>
        </CollapsibleGroup>
      </Show>

      {/* Raw tag count */}
      <Show when={props.data.rawTagCount}>
        <div class="text-[10px] text-txt-muted pt-1 border-t border-border/50">
          {props.data.rawTagCount} raw EXIF tags available
        </div>
      </Show>
    </div>
  );
}
