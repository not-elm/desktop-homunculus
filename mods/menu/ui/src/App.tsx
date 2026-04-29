import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@hmcs/ui';
import { useRef } from 'react';
import { useMenuActions } from './hooks/useMenuActions';
import { useMenuData } from './hooks/useMenuData';

export function App() {
  const { personaId, characterName, items } = useMenuData();
  const { closing, handleClose, handleSelect } = useMenuActions(personaId);
  const contentRef = useRef<HTMLDivElement>(null);

  if (items.length === 0) return null;

  const useGrid = items.length < 7;

  return (
    <DropdownMenu open={!closing} modal={false}>
      <DropdownMenuTrigger className="sr-only" />
      <DropdownMenuContent
        ref={contentRef}
        className="holo-refract-border holo-noise shadow-none relative w-[calc(100vw-20px)] max-w-[340px] overflow-x-hidden p-[var(--hud-space-sm)] [scrollbar-gutter:stable] animate-orbital-float [animation-delay:0.8s] data-[state=open]:animate-orbital-hud-open data-[state=closed]:animate-orbital-hud-closed after:content-[''] after:absolute after:left-[-6px] after:top-1/4 after:bottom-1/4 after:w-px after:rounded-[1px] after:pointer-events-none after:z-10 after:bg-[linear-gradient(180deg,transparent,oklch(0.72_0.14_var(--menu-accent-hue)/0.2)_25%,oklch(0.72_0.14_var(--menu-accent-hue)/0.35)_50%,oklch(0.72_0.14_var(--menu-accent-hue)/0.2)_75%,transparent)] motion-reduce:animate-none motion-reduce:data-[state=open]:animate-none motion-reduce:data-[state=closed]:animate-none motion-reduce:data-[state=open]:opacity-100"
        onEscapeKeyDown={handleClose}
        onPointerDownOutside={handleClose}
        sideOffset={0}
        align="start"
      >
        {/* Decorative layers */}
        <div className="menu-hud-highlight" />
        <div className="menu-hud-bottom-line" />
        <span className="menu-hud-corner menu-hud-corner--tl" />
        <span className="menu-hud-corner menu-hud-corner--tr" />
        <span className="menu-hud-corner menu-hud-corner--bl" />
        <span className="menu-hud-corner menu-hud-corner--br" />

        {/* Character status bar */}
        {characterName && (
          <>
            <div className="flex items-center justify-between px-[var(--hud-space-md)] pt-[var(--hud-space-xs)] pb-[var(--hud-space-sm)] relative z-[7]">
              <span className="text-[var(--hud-font-size-sm)] font-semibold tracking-[0.02em] text-[oklch(0.93_0.03_var(--menu-accent-hue))] overflow-hidden text-ellipsis whitespace-nowrap max-w-[200px]">
                {characterName}
              </span>
            </div>
            <div className="h-px mx-[var(--hud-space-sm)] mt-0.5 mb-[var(--hud-space-xs)] relative z-[7] bg-[linear-gradient(90deg,transparent,oklch(0.72_0.14_var(--menu-accent-hue)/0.2)_20%,oklch(0.65_0.18_285/0.15)_50%,oklch(0.7_0.16_350/0.2)_80%,transparent)]" />
          </>
        )}

        {/* Action card grid */}
        <div className={useGrid ? 'menu-card-grid' : 'menu-card-grid menu-card-grid--list'}>
          {items.map((item, i) => (
            <DropdownMenuItem
              key={item.id}
              className="menu-card menu-card-stagger"
              style={{ '--i': i } as React.CSSProperties}
              onSelect={() => handleSelect(item)}
            >
              <span className="menu-card-label">{item.text}</span>
            </DropdownMenuItem>
          ))}
        </div>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
