import * as RadioGroupPrimitive from '@radix-ui/react-radio-group';
import { useRef } from 'react';
import { cn, Input } from '@hmcs/ui';
import type { AgeValue } from '../PersonaFields';

interface AgeFieldProps {
  value: AgeValue;
  onChange: (age: AgeValue) => void;
  disabled?: boolean;
}

const segmentsClasses = cn(
  'flex h-8 gap-0.5 overflow-hidden rounded-md border bg-input',
  'border-primary/20',
  'data-[mode=unknown]:border-holo-violet/20',
);

const segmentClasses = cn(
  'flex flex-1 cursor-pointer items-center justify-center text-xs font-medium uppercase tracking-[0.08em] transition-colors',
  'text-muted-foreground',
  'hover:text-foreground',
  // active (data-state=checked) styling per data-mode
  'data-[state=checked][data-mode=specify]:bg-input/80 data-[state=checked][data-mode=specify]:text-primary data-[state=checked][data-mode=specify]:[text-shadow:0_0_12px_var(--primary)]',
  'data-[state=checked][data-mode=unknown]:bg-input/80 data-[state=checked][data-mode=unknown]:text-holo-violet data-[state=checked][data-mode=unknown]:[text-shadow:0_0_12px_var(--holo-violet)]',
  'disabled:cursor-not-allowed disabled:opacity-50',
);

const valueAreaClasses = cn(
  'mt-1.5 flex h-9 items-center rounded-md border px-3 transition-colors',
  'border-primary/20 focus-within:border-primary/50 focus-within:[box-shadow:0_0_8px_var(--primary)]',
  'data-[mode=unknown]:border-holo-violet/20 data-[mode=unknown]:focus-within:border-holo-violet/50',
);

export function AgeField({ value, onChange, disabled }: AgeFieldProps) {
  const preservedAge = useRef(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const mode = value.type;

  function handleModeChange(newMode: string) {
    if (disabled) return;
    if (newMode === 'unknown') {
      if (value.type === 'specify') preservedAge.current = value.age;
      onChange({ type: 'unknown' });
    } else {
      onChange({ type: 'specify', age: preservedAge.current });
      requestAnimationFrame(() => inputRef.current?.focus());
    }
  }

  function handleInput(raw: string) {
    const digits = raw.replace(/[^0-9]/g, '');
    if (digits === '') {
      onChange({ type: 'specify', age: 0 });
      return;
    }
    const age = parseInt(digits, 10);
    preservedAge.current = age;
    onChange({ type: 'specify', age });
  }

  return (
    <fieldset
      className="flex flex-col gap-1.5 text-xs uppercase tracking-[0.1em] text-primary/70"
      disabled={disabled}
    >
      <legend className="absolute -m-px h-px w-px overflow-hidden whitespace-nowrap [clip:rect(0,0,0,0)]">
        Age
      </legend>
      <RadioGroupPrimitive.Root
        className={segmentsClasses}
        value={mode}
        onValueChange={handleModeChange}
        data-mode={mode}
        disabled={disabled}
      >
        <RadioGroupPrimitive.Item
          value="specify"
          className={segmentClasses}
          aria-label="Specify age"
          data-mode="specify"
        >
          Specify
        </RadioGroupPrimitive.Item>
        <RadioGroupPrimitive.Item
          value="unknown"
          className={segmentClasses}
          aria-label="Age unknown"
          data-mode="unknown"
        >
          Unknown
        </RadioGroupPrimitive.Item>
      </RadioGroupPrimitive.Root>
      <div
        className={valueAreaClasses}
        role="status"
        aria-live="polite"
        data-mode={mode}
      >
        {mode === 'unknown' ? (
          <span className="font-mono text-sm text-holo-violet">Unknown</span>
        ) : (
          <Input
            ref={inputRef}
            type="text"
            inputMode="numeric"
            pattern="[0-9]*"
            size="sm"
            className="h-auto border-0 bg-transparent p-0 font-mono text-sm text-foreground shadow-none focus-visible:border-0 focus-visible:shadow-none focus-visible:ring-0"
            value={String(value.age)}
            onChange={(e) => handleInput(e.target.value)}
            aria-label="Age value"
            disabled={disabled}
          />
        )}
      </div>
    </fieldset>
  );
}
