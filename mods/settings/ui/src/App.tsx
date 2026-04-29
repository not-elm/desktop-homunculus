import { cn, Toolbar } from '@hmcs/ui';
import { GeneralTab } from './components/GeneralTab';
import { useSettings } from './hooks/useSettings';

const panelClasses =
  'relative box-border flex h-screen max-h-screen max-w-screen flex-col overflow-hidden rounded-xl bg-[oklch(0.15_0.01_250/0.92)] animate-settings-in motion-reduce:animate-none motion-reduce:opacity-100';

export function App() {
  const { loading, fps, setFps, alpha, setAlpha, handleClose } = useSettings();

  if (loading) {
    return (
      <div className={cn(panelClasses, 'items-center justify-center')}>
        <div className="text-xs uppercase tracking-[0.12em] text-[oklch(0.72_0.14_192/0.5)] animate-holo-corner-pulse-fast motion-reduce:animate-none motion-reduce:opacity-50">
          Loading...
        </div>
      </div>
    );
  }

  return (
    <div className={cn(panelClasses, 'holo-refract-border holo-noise')}>
      <div className="settings-highlight" />
      <div className="settings-bottom-line" />
      <div className="settings-scanline" />
      <span className="settings-corner settings-corner--tl" />
      <span className="settings-corner settings-corner--tr" />
      <span className="settings-corner settings-corner--bl" />
      <span className="settings-corner settings-corner--br" />

      <Toolbar title="Settings" onClose={handleClose} />

      <div className="no-scrollbar relative z-[7] flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-5">
        <GeneralTab fps={fps} setFps={setFps} alpha={alpha} setAlpha={setAlpha} />
      </div>
    </div>
  );
}
