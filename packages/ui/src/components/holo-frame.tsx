import type { HTMLAttributes, PropsWithChildren } from 'react';

import { cn } from '@/lib/utils';

/**
 * Animated diagonal light-sweep overlay.
 *
 * Renders as an absolutely-positioned, aria-hidden child rather than a
 * `::after` pseudo-element so it composes cleanly with other layered effects
 * (per Tailwind v4 guidance: prefer real elements over pseudo-elements).
 *
 * Inherits border-radius from parent so it matches a rounded card frame.
 *
 * @example
 * ```tsx
 * <div className="relative overflow-hidden rounded-xl">
 *   <HoloShimmer />
 *   <p>Content above the shimmer</p>
 * </div>
 * ```
 */
export function HoloShimmer({ className }: { className?: string }) {
  return (
    <span
      aria-hidden
      className={cn(
        'pointer-events-none absolute inset-0 overflow-hidden rounded-[inherit] holo-shimmer-bg',
        className,
      )}
    />
  );
}

/**
 * Subtle SVG fractal-noise overlay.
 *
 * Renders as an absolutely-positioned, aria-hidden child rather than a
 * `::before` pseudo-element. Opacity differs in light vs dark mode (handled
 * by the underlying `holo-noise-bg` utility).
 *
 * @example
 * ```tsx
 * <div className="relative rounded-xl">
 *   <HoloNoise />
 *   <p>Content above the noise</p>
 * </div>
 * ```
 */
export function HoloNoise({ className }: { className?: string }) {
  return (
    <span
      aria-hidden
      className={cn(
        'pointer-events-none absolute inset-0 z-[1] rounded-[inherit] holo-noise-bg',
        className,
      )}
    />
  );
}

export type HoloFrameProps = PropsWithChildren<HTMLAttributes<HTMLDivElement>>;

/**
 * Canonical "holo card" frame: animated refract-border + shimmer overlay +
 * noise overlay. This 3-effect stack is the foundation of the project's
 * glassmorphism design language — used together throughout `@hmcs/ui`
 * (Card, Drawer, Dialog) and mod UI shells.
 *
 * Children render in front of both overlays. Pass `className` to set
 * background, border-radius, padding, etc. on the wrapper.
 *
 * @example
 * ```tsx
 * <HoloFrame className="rounded-xl bg-card p-6 backdrop-blur-md">
 *   <h2>Settings</h2>
 *   <p>Content here</p>
 * </HoloFrame>
 * ```
 */
export function HoloFrame({ className, children, ...rest }: HoloFrameProps) {
  return (
    <div className={cn('relative overflow-hidden holo-refract-border', className)} {...rest}>
      <HoloShimmer />
      <HoloNoise />
      {children}
    </div>
  );
}
