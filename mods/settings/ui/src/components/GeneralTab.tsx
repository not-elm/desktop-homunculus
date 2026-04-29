interface GeneralTabProps {
  fps: number;
  setFps: (v: number) => void;
  alpha: number;
  setAlpha: (v: number) => void;
}

const labelClasses =
  'flex flex-col gap-1.5 text-xs uppercase tracking-[0.1em] text-[oklch(0.72_0.14_192/0.7)]';

const descriptionClasses =
  'text-[0.7rem] tracking-[0.04em] normal-case text-[oklch(0.55_0.02_250/0.6)] leading-[1.4]';

const sliderRowClasses = 'flex flex-row items-center gap-3';

const sliderValueClasses = 'min-w-[3.5em] text-right font-mono text-xs text-[oklch(0.72_0.14_192)]';

export function GeneralTab({ fps, setFps, alpha, setAlpha }: GeneralTabProps) {
  return (
    <div className="flex flex-col gap-4">
      <label className={labelClasses}>
        Frame Rate
        <div className={sliderRowClasses}>
          <input
            type="range"
            className="settings-slider"
            min={1}
            max={120}
            step={1}
            value={fps}
            onChange={(e) => setFps(Number(e.target.value))}
          />
          <span className={sliderValueClasses}>{Math.round(fps)} fps</span>
        </div>
        <span className={descriptionClasses}>
          Controls the rendering frame rate. Lower values reduce CPU/GPU usage.
        </span>
      </label>

      <label className={labelClasses}>
        Shadow Opacity
        <div className={sliderRowClasses}>
          <input
            type="range"
            className="settings-slider"
            min={0}
            max={1}
            step={0.01}
            value={alpha}
            onChange={(e) => setAlpha(Number(e.target.value))}
          />
          <span className={sliderValueClasses}>{Math.round(alpha * 100)}%</span>
        </div>
        <span className={descriptionClasses}>
          Controls the transparency of the shadow panel overlay behind the character.
        </span>
      </label>
    </div>
  );
}
