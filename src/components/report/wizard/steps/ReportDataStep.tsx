// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ReportDataStep - Router that renders the appropriate report-type-specific
 * data entry form based on the selected report type.
 */

import { Switch, Match } from "solid-js";
import { useWizard } from "../WizardContext";
import { COCFormSection } from "./reportdata/COCFormSection";
import { IARSchemaSection } from "./reportdata/IARSchemaSection";
import { UserActivitySchemaSection } from "./reportdata/UserActivitySchemaSection";
import { TimelineSchemaSection } from "./reportdata/TimelineSchemaSection";

export function ReportDataStep() {
  const ctx = useWizard();

  return (
    <div class="space-y-3">
      <Switch>
        <Match when={ctx.reportType() === "chain_of_custody"}>
          <COCFormSection />
        </Match>
        <Match when={ctx.reportType() === "investigative_activity"}>
          <IARSchemaSection />
        </Match>
        <Match when={ctx.reportType() === "user_activity"}>
          <UserActivitySchemaSection />
        </Match>
        <Match when={ctx.reportType() === "timeline"}>
          <TimelineSchemaSection />
        </Match>
      </Switch>
    </div>
  );
}
