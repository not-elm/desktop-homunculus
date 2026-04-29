/**
 * Decorative L-shaped corner accent rendered at one of the four corners of the
 * menu HUD. Each corner has its own color and animation stagger to create a
 * subtle rotating-pulse rhythm around the panel.
 */
type Position = 'tl' | 'tr' | 'bl' | 'br';

const positionClasses: Record<Position, string> = {
  tl: '-top-px -left-px border-t-[1.5px] border-l-[1.5px] border-t-[oklch(0.72_0.14_192/0.6)] border-l-[oklch(0.72_0.14_192/0.6)]',
  tr: '-top-px -right-px border-t-[1.5px] border-r-[1.5px] border-t-[oklch(0.65_0.18_285/0.4)] border-r-[oklch(0.65_0.18_285/0.4)] [animation-delay:0.75s]',
  bl: '-bottom-px -left-px border-b-[1.5px] border-l-[1.5px] border-b-[oklch(0.65_0.18_285/0.4)] border-l-[oklch(0.65_0.18_285/0.4)] [animation-delay:1.5s]',
  br: '-bottom-px -right-px border-b-[1.5px] border-r-[1.5px] border-b-[oklch(0.7_0.16_350/0.4)] border-r-[oklch(0.7_0.16_350/0.4)] [animation-delay:2.25s]',
};

export function HudCorner({ position }: { position: Position }) {
  return (
    <span
      className={`absolute w-2.5 h-2.5 pointer-events-none z-10 animate-holo-corner-pulse motion-reduce:animate-none motion-reduce:opacity-60 ${positionClasses[position]}`}
    />
  );
}
