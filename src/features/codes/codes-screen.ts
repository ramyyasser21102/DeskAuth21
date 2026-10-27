import { CodeView, listCodes, lockVault } from "../../lib/api";
import { el } from "../../lib/dom";

const REFRESH_INTERVAL_MS = 1000;
const IMPORT_HINT = "Import UI arrives in the next release; codes also appear in the tray menu.";

function buildCodeRow(view: CodeView): HTMLLIElement {
  const code = el("span", { className: "code" }, view.code);
  const secondsLeft = el("span", { className: "secs" }, `${view.seconds_left}s`);
  return el("li", {}, view.label, el("span", {}, code, secondsLeft));
}

/** Shows live codes. Returns a function that stops the refresh timer. */
export function renderCodesScreen(container: HTMLElement, onLocked: () => void): () => void {
  const list = el("ul");
  const lockButton = el("button", { className: "secondary", type: "button" }, "Lock");
  container.replaceChildren(el("h1", {}, "DeskAuth21"), list, el("p", { className: "hint" }, IMPORT_HINT), lockButton);

  const refresh = async () => {
    try {
      list.replaceChildren(...(await listCodes()).map(buildCodeRow));
    } catch {
      // Any failure here means the vault is no longer readable (usually locked
      // from the tray); hand control back so the router shows the right screen.
      stopRefreshing();
      onLocked();
    }
  };
  const timerId = window.setInterval(refresh, REFRESH_INTERVAL_MS);
  const stopRefreshing = () => window.clearInterval(timerId);

  lockButton.addEventListener("click", async () => {
    await lockVault();
    onLocked();
  });
  void refresh();
  return stopRefreshing;
}
