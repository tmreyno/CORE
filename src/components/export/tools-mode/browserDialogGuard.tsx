import { Show, createSignal, type Accessor } from "solid-js";
import { isTauri } from "../../../utils/platform";

const BROWSER_TOOLS_DIALOG_MESSAGE =
  "Archive tool file browsing is available in the desktop app. In browser preview, enter the path manually.";

export function createToolsBrowserDialogGuard() {
  const [message, setMessage] = createSignal("");

  const canUseNativeDialog = () => {
    if (!isTauri) {
      setMessage(BROWSER_TOOLS_DIALOG_MESSAGE);
      return false;
    }

    setMessage("");
    return true;
  };

  return { canUseNativeDialog, message };
}

export function ToolsBrowserDialogMessage(props: { message: Accessor<string> }) {
  return (
    <Show when={props.message()}>
      <div class="rounded border border-warning/30 bg-warning/10 px-3 py-2 text-xs text-warning">
        {props.message()}
      </div>
    </Show>
  );
}
