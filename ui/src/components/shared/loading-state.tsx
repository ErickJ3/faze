export function LoadingState() {
  return (
    <div className="flex items-center justify-center h-64">
      <div className="text-center">
        <div className="inline-block w-8 h-8 border-2 border-foreground/20 border-t-foreground rounded-full animate-spin mb-2" />
        <p className="text-sm text-foreground/50">Loading...</p>
      </div>
    </div>
  );
}
