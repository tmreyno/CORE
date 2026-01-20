/**
 * SettingGroup Component
 * Groups related settings with a title header
 */

import type { SettingGroupProps } from "./types";

/**
 * Container for grouping related settings
 */
export function SettingGroup(props: SettingGroupProps) {
  return (
    <div class="mb-6 last:mb-0">
      <h3 class="text-sm font-semibold text-txt mb-3">{props.title}</h3>
      <div class="flex flex-col gap-1">
        {props.children}
      </div>
    </div>
  );
}
