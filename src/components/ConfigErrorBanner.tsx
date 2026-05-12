type Props = {
  message: string;
  onReveal?: () => void;
  onRetry?: () => void;
};

export function ConfigErrorBanner({ message, onReveal, onRetry }: Props) {
  return (
    <div className="border-b border-red-800 bg-red-950/60 px-4 py-2 text-sm text-red-200">
      <div className="font-semibold">Config error</div>
      <div className="mt-1 whitespace-pre-wrap font-mono text-xs">{message}</div>
      <div className="mt-2 flex gap-2">
        {onReveal && (
          <button
            type="button"
            onClick={onReveal}
            className="rounded border border-red-700 px-2 py-1 text-xs hover:bg-red-900/50"
          >
            Open config folder
          </button>
        )}
        {onRetry && (
          <button
            type="button"
            onClick={onRetry}
            className="rounded border border-red-700 px-2 py-1 text-xs hover:bg-red-900/50"
          >
            Retry
          </button>
        )}
      </div>
    </div>
  );
}
