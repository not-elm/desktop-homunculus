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
        <div
          className={`grid gap-[5px] p-0.5 relative z-[7] ${useGrid ? 'grid-cols-2' : 'grid-cols-1'}`}
        >
          {items.map((item, i) => (
            <DropdownMenuItem
              key={item.id}
              className={`relative flex items-center rounded-md border cursor-pointer transition-all duration-150 ease-[cubic-bezier(0.4,0,0.2,1)] border-[oklch(1_0_0/0.14)] bg-[oklch(0.13_0.015_var(--menu-accent-hue)/0.45)] text-[oklch(0.9_0.01_230)] hover:border-[oklch(0.72_0.14_var(--menu-accent-hue)/0.5)] hover:bg-[linear-gradient(135deg,oklch(0.72_0.14_var(--menu-accent-hue)/0.18)_0%,oklch(0.65_0.18_285/0.08)_100%)] hover:-translate-y-px hover:text-[oklch(0.92_0.03_var(--menu-accent-hue))] focus:border-[oklch(0.72_0.14_var(--menu-accent-hue)/0.5)] focus:bg-[linear-gradient(135deg,oklch(0.72_0.14_var(--menu-accent-hue)/0.18)_0%,oklch(0.65_0.18_285/0.08)_100%)] focus:-translate-y-px focus:text-[oklch(0.92_0.03_var(--menu-accent-hue))] active:border-[oklch(0.65_0.18_285/0.4)] active:bg-[oklch(0.65_0.18_285/0.12)] active:scale-[0.97] active:text-[oklch(0.88_0.04_285)] data-[variant=destructive]:hover:border-[oklch(0.6_0.22_25/0.35)] data-[variant=destructive]:hover:bg-[oklch(0.6_0.22_25/0.1)] data-[variant=destructive]:hover:text-[oklch(0.78_0.16_25)] opacity-0 animate-orbital-card-in [animation-delay:calc(60ms+var(--i,0)*30ms)] motion-reduce:opacity-100 motion-reduce:animate-none ${useGrid ? 'min-h-[52px] justify-center px-2.5 py-[var(--hud-space-md)]' : 'min-h-[40px] justify-start px-[var(--hud-space-lg)] py-[var(--hud-space-sm)]'}`}
              style={{ '--i': i } as React.CSSProperties}
              onSelect={() => handleSelect(item)}
            >
              <span
                className={`text-[12.5px] font-medium tracking-[0.01em] leading-[1.3] pointer-events-none ${useGrid ? 'text-center' : 'text-left'}`}
              >
                {item.text}
              </span>
            </DropdownMenuItem>
          ))}
        </div>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
