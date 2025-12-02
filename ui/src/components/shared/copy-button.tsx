import { useState } from "react";
import { useToast } from "./toast-provider";

interface CopyButtonProps {
  text: string;
  label?: string;
}

export function CopyButton({ text, label = "Copy" }: CopyButtonProps) {
  const [copied, setCopied] = useState(false);
  const { showToast } = useToast();

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      showToast("Copied to clipboard", "success");
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      showToast("Failed to copy", "error");
      console.error("Failed to copy:", err);
    }
  };

  return (
    <button
      onClick={handleCopy}
      className="text-xs border border-border px-2 py-1 hover:bg-card transition-colors font-mono"
    >
      {copied ? "✓ Copied" : label}
    </button>
  );
}
