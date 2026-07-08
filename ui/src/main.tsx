import React from "react";
import ReactDOM from "react-dom/client";
import { RouterProvider } from "@tanstack/react-router";
import { QueryClientProvider } from "@tanstack/react-query";
import { NuqsAdapter } from "nuqs/adapters/react";
import { router } from "./router";
import { queryClient } from "./lib/query-client";
import { ToastProvider } from "./components/shared/toast-provider";
import { applyTheme, getTheme } from "./lib/theme";
import "./index.css";

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

applyTheme(getTheme());

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <NuqsAdapter>
        <ToastProvider>
          <RouterProvider router={router} />
        </ToastProvider>
      </NuqsAdapter>
    </QueryClientProvider>
  </React.StrictMode>,
);
