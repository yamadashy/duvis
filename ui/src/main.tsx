import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { loadStoredTheme } from "./state/theme";
import "./styles/tokens.css";
import "./styles/globals.css";

// Apply the stored theme before the first paint so the scanning splash and
// any pre-React UI doesn't flash the wrong colors.
document.documentElement.dataset.theme = loadStoredTheme();
document.documentElement.dataset.accent = "indigo";

const root = document.getElementById("root");
if (!root) throw new Error("#root not found");

createRoot(root).render(
  <StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </StrictMode>,
);
