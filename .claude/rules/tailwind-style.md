# Tailwind CSS Style

Mod UIs (`mods/*/ui/`) and `@hmcs/ui` use **Tailwind CSS v4**. All styling MUST follow Tailwind's utility-first methodology. Do not invent semantic class names, do not write parallel custom CSS, do not reach for `@apply` as a default.

## The Core Rule: Utility-First, Always

Style elements by composing utility classes directly in the markup. Do not author semantic classes like `.btn`, `.card`, `.modal-header`, `.glass-panel`, etc. and then style them in a CSS file.

```tsx
// ✅ Utility-first — styles live with the markup
<button className="rounded-md bg-primary/30 px-4 py-2 text-white backdrop-blur-sm hover:bg-primary/40">
  Save
</button>

// ❌ Semantic class + custom CSS
<button className="btn btn-primary">Save</button>
/* button.css */
.btn-primary { background: ...; padding: ...; }
```

**Why utility-first:**
- Adding/removing a utility only affects that one element — semantic classes create cross-page coupling and silent regressions.
- The CSS bundle stops growing linearly with features.
- Markup is portable: copy/paste a component to another mod and it brings its styles with it.
- No naming bikeshedding, no jumping between `.tsx` and `.css` files.

## Forbidden Patterns

- **Do NOT write component-scoped CSS files** (`Button.css`, `Settings.module.css`, etc.) for styling concerns Tailwind already covers. Delete them when found and migrate to utility classes.
- **Do NOT author semantic class names** as a styling layer (`.card`, `.btn-primary`, `.glass-panel`, `.section-header`). Class names that exist *only* to attach CSS rules are forbidden.
- **Do NOT use `@apply` to recreate semantic classes.** `@apply` is not a workaround that makes the previous two rules acceptable. Reuse comes from React components, not CSS classes (see *Handling Repetition* below).
- **Do NOT use inline `style={{ ... }}`** for static values. Inline styles cannot express hover/focus/responsive/dark variants and bypass the design system. They are acceptable *only* for dynamic values that come from runtime data (API responses, user-picked colors, computed positions).
- **Do NOT use native HTML elements (`<button>`, `<select>`, `<input>`) directly** in mod WebView UIs when a `@hmcs/ui` equivalent exists — see `ts-style.md`. Then style the `@hmcs/ui` component with utilities via `className`.

## Handling Repetition

When the same class list appears multiple times, **do not extract a CSS class**. Instead, in order of preference:

1. **Render in a loop.** If the duplication is "this list of N items all look the same", the class list is written once in the loop body — there is no duplication to solve.
2. **Extract a React component.** If a chunk of styled markup needs to be reused across files, make it a component (`<VacationCard />`, `<SettingRow />`). The component encapsulates *both* structure and Tailwind classes in one place. This is the canonical reuse mechanism.
3. **Multi-cursor edit.** For localized duplication within a single file, just edit all instances at once — extracting prematurely is worse than a few duplicated class lists.

`@apply` is a last resort, not a step in this list.

## Variants Are How You Express State and Context

Use Tailwind variants — never write custom CSS or use JS to toggle classes for these:

| Concern | Use |
|---|---|
| Hover / focus / active / disabled | `hover:`, `focus:`, `active:`, `disabled:` |
| Responsive breakpoints | `sm:`, `md:`, `lg:`, `xl:` (mobile-first) |
| Dark mode | `dark:` |
| Group / peer state | `group-hover:`, `peer-checked:`, etc. |
| Data attributes | `data-[state=open]:`, `aria-expanded:` |

```tsx
// ✅
<button className="bg-primary/30 hover:bg-primary/40 disabled:opacity-50 sm:px-6">…</button>

// ❌ — manual class toggling for what variants already do
<button className={`bg-primary/30 ${isHovered ? 'bg-primary/40' : ''}`} onMouseEnter={…}>…</button>
```

Variants can stack: `dark:hover:bg-primary/40`, `sm:focus-visible:ring-2`.

## Arbitrary Values: Bracket Syntax, Not Custom CSS

When you need a one-off value not in the theme, use bracket syntax — do not drop down to a CSS file:

```tsx
<div className="bg-[#316ff6]" />
<div className="grid-cols-[24rem_2.5rem_minmax(0,1fr)]" />
<div className="max-h-[calc(100dvh-theme(spacing.6))]" />
<div className="[--gutter:1rem] lg:[--gutter:2rem]" />
```

If the same arbitrary value appears in 3+ places, promote it to the theme (a CSS variable in the Tailwind config / `@theme` block) instead of repeating the brackets.

## Project Design Language

Mod WebView UIs follow the **glassmorphism** look documented in the root `CLAUDE.md`:
- Semi-transparent backgrounds: `bg-primary/30`, `bg-white/10`
- Backdrop blur: `backdrop-blur-sm`
- Subtle borders: `border border-white/20`
- White text on the transparent Bevy window: `text-white`

Compose these with utilities. Do not create a `.glass` class.

## Conditional Classes: Use `cn()`

For conditional class composition, use the `cn()` helper exported from `@hmcs/ui` (clsx + tailwind-merge). It correctly resolves conflicts (e.g., `px-4` vs `px-6`).

```tsx
import { cn } from "@hmcs/ui";

<div className={cn(
  "rounded-md px-4 py-2",
  isActive && "bg-primary/40",
  variant === "ghost" && "bg-transparent",
  className, // allow caller overrides
)} />
```

Do not hand-roll string concatenation (`` `${a} ${b ? c : ''}` ``) when `cn()` is available.

## When Custom CSS Is Acceptable

Limited, narrow scenarios — not as an escape hatch from utility-first:

1. **Dynamic values from runtime data** (theme picker, user-supplied color, computed pixel position): use inline `style={{ … }}` for the dynamic property *only*, and keep everything else as utility classes.
2. **Genuinely global concerns** that cannot be expressed per-element: CSS resets, font-face declarations, scrollbar theming, base typography. These belong in the single global stylesheet, not per-component.
3. **Animations/keyframes** that Tailwind does not provide out of the box: define `@keyframes` once globally, then reference via `animate-[name]` utilities.

If you find yourself wanting custom CSS for anything else, the answer is almost always "extract a React component" instead.

## Review Checklist

Before submitting UI changes:
- [ ] No new `.css` / `.module.css` files for component-level styling.
- [ ] No new semantic class names (`.btn-*`, `.card-*`, `.glass-*`).
- [ ] No `@apply` directives added.
- [ ] No inline `style={{ … }}` for static values.
- [ ] Hover/focus/responsive/dark are expressed as variants, not JS state or media-query CSS.
- [ ] Repeated markup is reused via a React component, not a shared class.
- [ ] `cn()` used for conditional classes.
