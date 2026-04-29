/**
 * Decorative HUD layers for the VOICEVOX settings panel: a top-edge highlight
 * with shimmering sweep, a bottom-edge fade line, an animated scanline, and
 * four animated corner accents. All elements are aria-hidden and pointer-events
 * disabled so they don't interfere with focus order or click-through.
 */
export function Decorations() {
  return (
    <>
      <HudHighlight />
      <div
        aria-hidden="true"
        className="pointer-events-none absolute bottom-0 left-[10%] right-[10%] z-10 h-px bg-[linear-gradient(90deg,transparent,oklch(0.7_0.16_350/0.15)_30%,oklch(0.65_0.18_285/0.1)_50%,oklch(0.72_0.14_192/0.15)_70%,transparent)]"
      />
      <HudScanline />
      <HudCorner position="tl" />
      <HudCorner position="tr" />
      <HudCorner position="bl" />
      <HudCorner position="br" />
    </>
  );
}

/**
 * Top-edge highlight bar with a multi-stop holographic gradient and an
 * animated shimmer sweep overlay.
 */
function HudHighlight() {
  return (
    <div
      aria-hidden="true"
      className="pointer-events-none absolute left-0 right-0 top-0 z-10 h-px rounded-[inherit] bg-[linear-gradient(90deg,transparent_5%,oklch(0.72_0.14_192/0.35)_20%,oklch(0.65_0.18_285/0.25)_50%,oklch(0.7_0.16_350/0.35)_80%,transparent_95%)] after:absolute after:inset-0 after:bg-[linear-gradient(90deg,transparent_0%,oklch(1_0_0/0.15)_45%,oklch(1_0_0/0.25)_50%,oklch(1_0_0/0.15)_55%,transparent_100%)] after:bg-[length:200%_100%] after:[content:''] after:animate-holo-highlight-sweep after:[animation-delay:1s] motion-reduce:after:animate-none"
    />
  );
}

/**
 * Vertically-traveling scanline overlay creating a CRT-like sweep effect over
 * the panel.
 */
function HudScanline() {
  return (
    <div
      aria-hidden="true"
      className="pointer-events-none absolute inset-0 z-[6] overflow-hidden rounded-[inherit] after:absolute after:left-0 after:right-0 after:h-10 after:bg-[linear-gradient(180deg,transparent,oklch(0.72_0.14_192/0.03)_40%,oklch(0.72_0.14_192/0.05)_50%,oklch(0.72_0.14_192/0.03)_60%,transparent)] after:[content:''] after:animate-holo-scanline motion-reduce:after:animate-none"
    />
  );
}

type Position = 'tl' | 'tr' | 'bl' | 'br';

const positionClasses: Record<Position, string> = {
  tl: '-top-px -left-px border-t-[1.5px] border-l-[1.5px] border-t-[oklch(0.72_0.14_192/0.6)] border-l-[oklch(0.72_0.14_192/0.6)] [filter:drop-shadow(0_0_3px_oklch(0.72_0.14_192/0.3))]',
  tr: '-top-px -right-px border-t-[1.5px] border-r-[1.5px] border-t-[oklch(0.65_0.18_285/0.4)] border-r-[oklch(0.65_0.18_285/0.4)] [filter:drop-shadow(0_0_3px_oklch(0.65_0.18_285/0.2))] [animation-delay:0.75s]',
  bl: '-bottom-px -left-px border-b-[1.5px] border-l-[1.5px] border-b-[oklch(0.65_0.18_285/0.4)] border-l-[oklch(0.65_0.18_285/0.4)] [filter:drop-shadow(0_0_3px_oklch(0.65_0.18_285/0.2))] [animation-delay:1.5s]',
  br: '-bottom-px -right-px border-b-[1.5px] border-r-[1.5px] border-b-[oklch(0.7_0.16_350/0.4)] border-r-[oklch(0.7_0.16_350/0.4)] [filter:drop-shadow(0_0_3px_oklch(0.7_0.16_350/0.2))] [animation-delay:2.25s]',
};

/**
 * Decorative L-shaped corner accent rendered at one of the four panel corners.
 * Each corner has its own color and animation stagger.
 */
function HudCorner({ position }: { position: Position }) {
  return (
    <span
      aria-hidden="true"
      className={`pointer-events-none absolute z-10 h-3 w-3 animate-holo-corner-pulse motion-reduce:animate-none motion-reduce:opacity-60 ${positionClasses[position]}`}
    />
  );
}
