import { el } from "../../lib/dom";
import { findPassphraseProblem } from "./passphrase-rules";

export interface PassphraseFormOptions {
  title: string;
  hint?: string;
  submitLabel: string;
  requiresConfirmation: boolean;
  onSubmit: (passphrase: string) => Promise<void>;
}

const createPasswordInput = (placeholder: string) =>
  el("input", { type: "password", placeholder, autocomplete: "off" });

export function renderPassphraseForm(container: HTMLElement, options: PassphraseFormOptions): void {
  const passphraseInput = createPasswordInput("Passphrase");
  const confirmationInput = createPasswordInput("Confirm passphrase");
  const errorMessage = el("p", { className: "error" });
  const fields = options.requiresConfirmation ? [passphraseInput, confirmationInput] : [passphraseInput];
  const form = el("form", {}, ...fields, errorMessage, el("button", { type: "submit" }, options.submitLabel));

  form.addEventListener("submit", (event) => {
    event.preventDefault();
    void submitForm(options, passphraseInput.value, confirmationInput.value, errorMessage);
  });

  const hint = options.hint ? [el("p", { className: "hint" }, options.hint)] : [];
  container.replaceChildren(el("h1", {}, options.title), ...hint, form);
  passphraseInput.focus();
}

async function submitForm(
  options: PassphraseFormOptions,
  passphrase: string,
  confirmation: string,
  errorMessage: HTMLElement,
): Promise<void> {
  errorMessage.textContent = findPassphraseProblem(passphrase, confirmation, options.requiresConfirmation);
  if (errorMessage.textContent) return;
  try {
    await options.onSubmit(passphrase);
  } catch (error) {
    errorMessage.textContent = String(error);
  }
}
