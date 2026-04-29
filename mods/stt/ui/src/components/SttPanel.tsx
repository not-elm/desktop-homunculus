import { audio, Webview } from '@hmcs/sdk';
import { cn, Toolbar } from '@hmcs/ui';
import { useCallback } from 'react';
import { type ModelCardState, useStt } from '../hooks/useStt';

export function SttPanel() {
  const { models, downloadModel, cancelDownload, errorMessage } = useStt();

  const handleClose = useCallback(() => {
    audio.se.play('se:close');
    Webview.current()?.close();
  }, []);

  return (
    <div className="holo-refract-border holo-noise relative box-border flex h-screen max-h-screen max-w-screen flex-col overflow-hidden rounded-xl bg-[oklch(0.15_0.01_250/0.92)] animate-settings-in motion-reduce:animate-none motion-reduce:opacity-100">
      {/* Decorative elements */}
      <div className="settings-highlight" />
      <div className="settings-bottom-line" />
      <div className="settings-scanline" />
      <span className="settings-corner settings-corner--tl" />
      <span className="settings-corner settings-corner--tr" />
      <span className="settings-corner settings-corner--bl" />
      <span className="settings-corner settings-corner--br" />

      <Toolbar title="Speech to Text" onClose={handleClose} />

      <div className="relative z-[7] flex min-h-0 flex-1 flex-col gap-[var(--hud-space-xl)] overflow-y-auto p-5 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
        <div className="flex flex-col gap-[var(--hud-space-xl)]">
          {errorMessage && (
            <div className="py-[var(--hud-space-sm)] text-[0.7rem] tracking-[0.04em] text-[oklch(0.7_0.18_25/0.8)]">
              {errorMessage}
            </div>
          )}

          <div className="mt-[var(--hud-space-xs)] flex items-center gap-[var(--hud-space-md)]">
            <span className="whitespace-nowrap text-[0.7rem] uppercase tracking-[0.12em] text-[oklch(0.55_0.02_250/0.5)]">
              Models
            </span>
            <span className="h-px flex-1 bg-[linear-gradient(90deg,oklch(0.72_0.14_192/0.15),transparent)]" />
          </div>

          <div className="grid grid-cols-2 gap-[var(--hud-space-md)]">
            {models.map((model) => (
              <ModelCard
                key={model.size}
                model={model}
                onDownload={() => downloadModel(model.size)}
                onCancel={() => cancelDownload(model.size)}
              />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

function ModelCard({
  model,
  onDownload,
  onCancel,
}: {
  model: ModelCardState;
  onDownload: () => void;
  onCancel: () => void;
}) {
  const isReady = model.status === 'downloaded';
  const isDownloading = model.status === 'downloading';

  return (
    <div
      className={cn(
        'flex flex-col gap-[var(--hud-space-sm)] rounded-lg border p-[var(--hud-space-lg)] text-left font-[inherit] bg-[oklch(0.18_0.01_250/0.6)]',
        isReady ? 'border-[oklch(0.72_0.14_192/0.3)]' : 'border-[oklch(0.4_0.02_250/0.25)]',
      )}
      title={`${model.label} model, ${model.fileSize}${
        isReady ? ', ready' : isDownloading ? ', downloading' : ', not downloaded'
      }`}
    >
      <span className="text-[0.8rem] font-semibold uppercase tracking-[0.08em] text-[oklch(0.85_0.04_250)]">
        {model.label}
      </span>
      <span className="font-mono text-[0.7rem] text-[oklch(0.55_0.02_250/0.6)]">
        {model.fileSize}
      </span>

      {model.status === 'not_downloaded' && (
        <button
          type="button"
          className="inline-block cursor-pointer rounded border border-[oklch(0.72_0.14_192/0.25)] bg-[oklch(0.72_0.14_192/0.1)] px-[var(--hud-space-md)] py-[var(--hud-space-xs)] font-[inherit] text-[0.7rem] uppercase tracking-[0.06em] text-[oklch(0.72_0.14_192)] transition-[background,border-color] duration-[var(--hud-duration-content)] ease-[ease] hover:border-[oklch(0.72_0.14_192/0.4)] hover:bg-[oklch(0.72_0.14_192/0.2)]"
          onClick={onDownload}
        >
          Download
        </button>
      )}

      {isDownloading && (
        <>
          <div className="h-1 overflow-hidden rounded-sm bg-[oklch(0.25_0.01_250/0.4)]">
            <div
              className="h-full rounded-sm bg-[linear-gradient(90deg,oklch(0.72_0.14_192),oklch(0.65_0.18_285))] transition-[width] duration-[var(--hud-duration-content)] ease-[ease]"
              style={{ width: `${model.downloadProgress ?? 0}%` }}
            />
          </div>
          <div className="flex items-center justify-between">
            <span className="font-mono text-[0.7rem] text-[oklch(0.55_0.02_250/0.6)]">
              {Math.round(model.downloadProgress ?? 0)}%
            </span>
            <button
              type="button"
              className="cursor-pointer border-none bg-transparent px-[var(--hud-space-xs)] py-0.5 text-[var(--hud-font-size-sm)] text-[oklch(0.55_0.02_250/0.5)] hover:text-[oklch(0.7_0.18_25)]"
              onClick={onCancel}
            >
              ✕
            </button>
          </div>
        </>
      )}

      {isReady && (
        <span className="text-[0.7rem] tracking-[0.04em] text-[oklch(0.65_0.18_145)]">✓ Ready</span>
      )}
    </div>
  );
}
