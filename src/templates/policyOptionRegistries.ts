import {
  CORE_BASELINE_PACKAGING_TYPE_OPTIONS,
  CORE_BASELINE_STORAGE_CLASS_OPTIONS,
  CORE_BASELINE_TRANSFER_METHOD_OPTIONS,
  CORE_BASELINE_TRANSFER_PURPOSE_OPTIONS,
} from "@core-suite/types/evidence-policy";
import type { InlineOption, OptionRegistry } from "./types";

function createPolicyOptionRegistry(
  id: string,
  name: string,
  items: readonly InlineOption[],
): OptionRegistry {
  return {
    id,
    name,
    version: "core-baseline",
    items: [...items],
  };
}

export const POLICY_OPTION_REGISTRIES: Record<string, OptionRegistry> = {
  coc_transfer_methods: createPolicyOptionRegistry(
    "coc_transfer_methods",
    "COC Transfer Methods",
    CORE_BASELINE_TRANSFER_METHOD_OPTIONS,
  ),
  coc_transfer_purposes: createPolicyOptionRegistry(
    "coc_transfer_purposes",
    "COC Transfer Purposes",
    CORE_BASELINE_TRANSFER_PURPOSE_OPTIONS,
  ),
  packaging_types: createPolicyOptionRegistry(
    "packaging_types",
    "Packaging Types",
    CORE_BASELINE_PACKAGING_TYPE_OPTIONS,
  ),
  storage_classes: createPolicyOptionRegistry(
    "storage_classes",
    "Storage Classes",
    CORE_BASELINE_STORAGE_CLASS_OPTIONS,
  ),
};