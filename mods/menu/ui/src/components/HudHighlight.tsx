/**
 * Top-edge highlight for the menu HUD. Renders a subtle multi-stop holographic
 * gradient line, with an animated white-noise shimmer sweeping across it via a
 * pseudo-element overlay.
 */
export function HudHighlight() {
  return (
    <div
      aria-hidden
      className="absolute top-0 left-0 right-0 h-px pointer-events-none z-10 rounded-[inherit] bg-[linear-gradient(90deg,transparent_5%,oklch(0.72_0.14_192/0.35)_20%,oklch(0.65_0.18_285/0.25)_50%,oklch(0.7_0.16_350/0.35)_80%,transparent_95%)] after:content-[''] after:absolute after:inset-0 after:bg-[linear-gradient(90deg,transparent_0%,oklch(1_0_0/0.15)_45%,oklch(1_0_0/0.25)_50%,oklch(1_0_0/0.15)_55%,transparent_100%)] after:bg-[length:200%_100%] after:animate-holo-highlight-sweep after:[animation-delay:1s] motion-reduce:after:animate-none"
    />
  );
}
