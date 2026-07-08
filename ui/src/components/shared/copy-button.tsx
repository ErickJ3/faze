import { useState } from "react";
import { useToast } from "./toast-provider";
import { Button } from "@/components/ui/button";

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
    <Button
      variant="outline"
      size="sm"
      onClick={handleCopy}
      aria-label={label}
      className="font-mono text-xs"
    >
      {copied ? "✓ Copied" : label}
    </Button>
  );
}
