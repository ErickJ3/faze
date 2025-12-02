import { createContext, useContext, useState, useCallback } from "react";
import type { ReactNode } from "react";

interface Toast {
  id: string;
  message: string;
  type: "success" | "error" | "info";
}

interface ToastContextType {
  toasts: Toast[];
  showToast: (message: string, type?: Toast["type"]) => void;
  hideToast: (id: string) => void;
}

const ToastContext = createContext<ToastContextType | undefined>(undefined);

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const showToast = useCallback(
    (message: string, type: Toast["type"] = "info") => {
      const id = Math.random().toString(36).substring(7);
      const toast: Toast = { id, message, type };

      setToasts((prev) => [...prev, toast]);

      setTimeout(() => {
        setToasts((prev) => prev.filter((t) => t.id !== id));
      }, 3000);
    },
    [],
  );

  const hideToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  return (
    <ToastContext.Provider value={{ toasts, showToast, hideToast }}>
      {children}
      <ToastContainer toasts={toasts} onClose={hideToast} />
    </ToastContext.Provider>
  );
}

function ToastContainer({
  toasts,
  onClose,
}: {
  toasts: Toast[];
  onClose: (id: string) => void;
}) {
  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 min-w-[300px]">
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={`
            border border-border p-4 bg-background
            animate-in slide-in-from-right
            ${toast.type === "error" ? "border-red-500" : ""}
            ${toast.type === "success" ? "border-green-500" : ""}
          `}
        >
          <div className="flex items-start justify-between gap-3">
            <p className="text-sm font-mono flex-1">{toast.message}</p>
            <button
              onClick={() => onClose(toast.id)}
              className="text-foreground/50 hover:text-foreground text-xs"
            >
              ✕
            </button>
          </div>
        </div>
      ))}
    </div>
  );
}

export function useToast() {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error("useToast must be used within ToastProvider");
  }
  return context;
}
