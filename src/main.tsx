import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./app";
import { PrivacySettingsProvider } from "./features/privacy/privacy-settings-provider";
import { ProfileProvider } from "./features/profile/profile-provider";
import { QuickPanel } from "./features/quick-panel/quick-panel";
import { isPanelView } from "./features/quick-panel/panel-keys";
import { ThemeProvider } from "./features/theme/theme-provider";
import { applyStoredTheme } from "./features/theme/theme-preference";
import "./index.css";

applyStoredTheme();

/** The menu bar panel and the main window are the same page; the address says which to show. */
const view = isPanelView(window.location.search) ? (
  <PrivacySettingsProvider>
    <QuickPanel />
  </PrivacySettingsProvider>
) : (
  <ProfileProvider>
    <App />
  </ProfileProvider>
);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ThemeProvider>{view}</ThemeProvider>
  </StrictMode>,
);
