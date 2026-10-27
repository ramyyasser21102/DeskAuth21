import { StrictMode } from "react";
import { MotionConfig } from "motion/react";
import { createRoot } from "react-dom/client";
import { App } from "./app";
import { ProfileProvider } from "./features/profile/profile-provider";
import { ThemeProvider } from "./features/theme/theme-provider";
import { applyStoredTheme } from "./features/theme/theme-preference";
import "./index.css";

applyStoredTheme();

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <MotionConfig reducedMotion="user">
      <ThemeProvider>
        <ProfileProvider>
          <App />
        </ProfileProvider>
      </ThemeProvider>
    </MotionConfig>
  </StrictMode>,
);
