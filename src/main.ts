import { invoke } from "@tauri-apps/api/core";

type VaultStatus = "uninitialized" | "locked" | "unlocked";

interface CodeView {
  label: string;
  code: string;
  seconds_left: number;
}

const app = document.querySelector<HTMLElement>("#app")!;
let timer: number | undefined;

function stopTimer() {
  if (timer !== undefined) clearInterval(timer);
  timer = undefined;
}

function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  props: Partial<HTMLElementTagNameMap[K]> = {},
  ...children: (Node | string)[]
): HTMLElementTagNameMap[K] {
  const node = Object.assign(document.createElement(tag), props);
  node.append(...children);
  return node;
}

function passphraseForm(opts: {
  title: string;
  hint?: string;
  submitLabel: string;
  confirm: boolean;
  onSubmit: (passphrase: string) => Promise<void>;
}) {
  const pass = el("input", { type: "password", placeholder: "Passphrase", autocomplete: "off" });
  const again = el("input", { type: "password", placeholder: "Confirm passphrase", autocomplete: "off" });
  const error = el("p", { className: "error" });
  const form = el("form", {}, pass, ...(opts.confirm ? [again] : []), error, el("button", { type: "submit" }, opts.submitLabel));
  form.addEventListener("submit", async (e) => {
    e.preventDefault();
    error.textContent = "";
    if (opts.confirm && pass.value !== again.value) {
      error.textContent = "Passphrases do not match.";
      return;
    }
    try {
      await opts.onSubmit(pass.value);
    } catch (err) {
      error.textContent = String(err);
    }
  });
  app.replaceChildren(el("h1", {}, opts.title), ...(opts.hint ? [el("p", { className: "hint" }, opts.hint)] : []), form);
  pass.focus();
}

async function renderCodes() {
  const list = el("ul");
  const draw = async () => {
    try {
      const codes = await invoke<CodeView[]>("list_codes");
      list.replaceChildren(
        ...codes.map((c) =>
          el("li", {}, c.label, el("span", {}, el("span", { className: "code" }, c.code), el("span", { className: "secs" }, `${c.seconds_left}s`))),
        ),
      );
    } catch {
      stopTimer();
      await render();
    }
  };
  const lockBtn = el("button", { className: "secondary", type: "button" }, "Lock");
  lockBtn.addEventListener("click", async () => {
    await invoke("lock");
    await render();
  });
  app.replaceChildren(
    el("h1", {}, "DeskAuth21"),
    list,
    el("p", { className: "hint" }, "Import UI arrives in the next release; codes also appear in the tray menu."),
    lockBtn,
  );
  await draw();
  timer = window.setInterval(draw, 1000);
}

async function render() {
  stopTimer();
  const status = await invoke<VaultStatus>("vault_status");
  if (status === "uninitialized") {
    passphraseForm({
      title: "Create your vault",
      hint: "Choose a passphrase of at least 8 characters. It cannot be recovered if you forget it.",
      submitLabel: "Create vault",
      confirm: true,
      onSubmit: async (passphrase) => {
        await invoke("create_vault", { passphrase });
        await render();
      },
    });
  } else if (status === "locked") {
    passphraseForm({
      title: "Unlock DeskAuth21",
      submitLabel: "Unlock",
      confirm: false,
      onSubmit: async (passphrase) => {
        await invoke("unlock", { passphrase });
        await render();
      },
    });
  } else {
    await renderCodes();
  }
}

render();
