import {
  cn,
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
  Textarea,
} from '@hmcs/ui';
import { useState } from 'react';
import type { VoicevoxSettings } from './hooks/useVoicevoxSettings';
import { useVoicevoxSettings } from './hooks/useVoicevoxSettings';

const PARAMS: {
  key: keyof VoicevoxSettings;
  label: string;
  desc: string;
  min: number;
  max: number;
  step: number;
}[] = [
  {
    key: 'speedScale',
    label: 'Speed',
    desc: 'Speech speed',
    min: 0.5,
    max: 2.0,
    step: 0.05,
  },
  {
    key: 'pitchScale',
    label: 'Pitch',
    desc: 'Voice pitch (0 = baseline)',
    min: -0.15,
    max: 0.15,
    step: 0.01,
  },
  {
    key: 'intonationScale',
    label: 'Intonation',
    desc: 'Intonation strength',
    min: 0.0,
    max: 2.0,
    step: 0.05,
  },
  {
    key: 'volumeScale',
    label: 'Volume',
    desc: 'Volume level',
    min: 0.0,
    max: 2.0,
    step: 0.05,
  },
  {
    key: 'pauseLength',
    label: 'Pause Length',
    desc: 'Pause duration at punctuation marks',
    min: 0,
    max: 2.0,
    step: 0.01,
  },
  {
    key: 'prePhonemeLength',
    label: 'Pre Phoneme Length',
    desc: 'Silence before speech starts',
    min: 0,
    max: 1.5,
    step: 0.01,
  },
  {
    key: 'postPhonemeLength',
    label: 'Post Phoneme Length',
    desc: 'Silence after speech ends',
    min: 0,
    max: 1.5,
    step: 0.01,
  },
];

export function App() {
  const {
    loading,
    connected,
    speakers,
    settings,
    setSettings,
    saving,
    saved,
    characterName,
    invalidSpeaker,
    personaId,
    speaking,
    handleSave,
    handleReset,
    handleClose,
    handleRetry,
    handleSpeak,
  } = useVoicevoxSettings();

  if (loading) {
    return (
      <div className="holo-refract-border holo-noise relative box-border flex h-screen max-h-screen max-w-screen flex-col gap-[var(--hud-space-xl)] overflow-hidden rounded-xl bg-[oklch(0.15_0.01_250/0.92)] p-5 animate-settings-in motion-reduce:animate-none motion-reduce:opacity-100 items-center justify-center min-h-[200px]">
        <div className="text-[var(--hud-font-size-sm)] uppercase tracking-[0.12em] text-[oklch(0.72_0.14_192/0.5)] animate-[holo-corner-pulse_2s_ease-in-out_infinite] motion-reduce:animate-none motion-reduce:opacity-50">
          Loading...
        </div>
      </div>
    );
  }

  return (
    <div className="holo-refract-border holo-noise relative box-border flex h-screen max-h-screen max-w-screen flex-col gap-[var(--hud-space-xl)] overflow-hidden rounded-xl bg-[oklch(0.15_0.01_250/0.92)] p-5 animate-settings-in motion-reduce:animate-none motion-reduce:opacity-100">
      <Decorations />
      <Header name={characterName} connected={connected} />

      <div className="relative z-[7] min-h-0 flex-1 overflow-y-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
        {!connected ? (
          <DisconnectedView onRetry={handleRetry} />
        ) : speakers.length === 0 ? (
          <div className="flex flex-col items-center justify-center gap-[var(--hud-space-xl)] px-[var(--hud-space-2xl)] py-10 text-center">
            <div className="text-[var(--hud-font-size-base)] text-[oklch(0.7_0.16_350/0.8)]">
              No speakers found
            </div>
          </div>
        ) : (
          <SettingsForm
            speakers={speakers}
            settings={settings}
            onSettingsChange={setSettings}
            invalidSpeaker={invalidSpeaker}
            speaking={speaking}
            disabled={!connected || speakers.length === 0 || invalidSpeaker || !personaId}
            onSpeak={handleSpeak}
          />
        )}
      </div>

      <Footer
        onClose={handleClose}
        onReset={handleReset}
        onSave={handleSave}
        saving={saving}
        saved={saved}
        disabled={!connected || speakers.length === 0 || invalidSpeaker}
      />
    </div>
  );
}

function Decorations() {
  return (
    <>
      <div className="settings-highlight" />
      <div className="settings-bottom-line" />
      <div className="settings-scanline" />
      <span className="settings-corner settings-corner--tl" />
      <span className="settings-corner settings-corner--tr" />
      <span className="settings-corner settings-corner--bl" />
      <span className="settings-corner settings-corner--br" />
    </>
  );
}

function Header({ name, connected }: { name: string; connected: boolean }) {
  return (
    <div className="relative z-[7] flex flex-row items-center justify-between">
      <h1 className="m-0 text-[var(--hud-font-size-base)] font-semibold uppercase tracking-[0.15em] text-[oklch(0.72_0.14_192/0.9)]">
        VOICEVOX
      </h1>
      <span className="font-mono text-[var(--hud-font-size-sm)] text-[oklch(0.72_0.14_192/0.5)]">
        {name}
      </span>
      <span
        className={cn(
          'ml-auto inline-flex items-center gap-[var(--hud-space-sm)] rounded-[20px] border px-[10px] py-[3px] text-[var(--hud-font-size-xs)]',
          connected
            ? 'border-[oklch(0.65_0.18_145/0.2)] bg-[oklch(0.65_0.18_145/0.1)] text-[oklch(0.65_0.18_145/0.8)]'
            : 'border-[oklch(0.7_0.16_350/0.2)] bg-[oklch(0.7_0.16_350/0.1)] text-[oklch(0.7_0.16_350/0.8)]',
        )}
      >
        <span
          className={cn(
            'h-1.5 w-1.5 rounded-full',
            connected
              ? 'bg-[oklch(0.65_0.18_145)] [box-shadow:0_0_6px_oklch(0.65_0.18_145/0.5)]'
              : 'bg-[oklch(0.7_0.16_350)] [box-shadow:0_0_6px_oklch(0.7_0.16_350/0.5)]',
          )}
        />
        {connected ? 'Connected' : 'Disconnected'}
      </span>
    </div>
  );
}

function DisconnectedView({ onRetry }: { onRetry: () => void }) {
  return (
    <div className="flex flex-col items-center justify-center gap-[var(--hud-space-xl)] px-[var(--hud-space-2xl)] py-10 text-center">
      <div className="text-[var(--hud-font-size-base)] text-[oklch(0.7_0.16_350/0.8)]">
        Cannot connect to VOICEVOX
      </div>
      <button
        type="button"
        className="relative z-[7] cursor-pointer rounded-md border border-[oklch(0.72_0.14_192/0.3)] bg-[oklch(0.72_0.14_192/0.2)] px-5 py-[var(--hud-space-md)] text-[13px] font-medium text-[oklch(0.72_0.14_192)] transition-all duration-[var(--hud-duration-content)] ease-out hover:bg-[oklch(0.72_0.14_192/0.3)]"
        onClick={onRetry}
      >
        Retry
      </button>
    </div>
  );
}

function SettingsForm({
  speakers,
  settings,
  onSettingsChange,
  invalidSpeaker,
  speaking,
  disabled,
  onSpeak,
}: {
  speakers: { name: string; styles: { name: string; id: number }[] }[];
  settings: VoicevoxSettings;
  onSettingsChange: (s: VoicevoxSettings) => void;
  invalidSpeaker: boolean;
  speaking: boolean;
  disabled: boolean;
  onSpeak: (text: string) => void;
}) {
  return (
    <>
      {invalidSpeaker && (
        <div className="relative z-[7] rounded-md border border-[oklch(0.8_0.14_80/0.2)] bg-[oklch(0.8_0.14_80/0.08)] px-[var(--hud-space-lg)] py-[var(--hud-space-md)] text-[var(--hud-font-size-sm)] text-[oklch(0.8_0.14_80/0.8)]">
          Previous speaker is unavailable. Please select a new one.
        </div>
      )}

      <label
        className="flex flex-col gap-[var(--hud-space-sm)] text-[var(--hud-font-size-sm)] uppercase tracking-[0.1em] text-[oklch(0.72_0.14_192/0.7)]"
        htmlFor="voicevox-speaker-select"
      >
        Speaker
        <Select
          value={settings.speakerId === -1 ? undefined : String(settings.speakerId)}
          onValueChange={(value) => onSettingsChange({ ...settings, speakerId: Number(value) })}
        >
          <SelectTrigger id="voicevox-speaker-select" className="w-full">
            <SelectValue placeholder="— Select a speaker —" />
          </SelectTrigger>
          <SelectContent>
            {speakers.map((speaker) => (
              <SelectGroup key={speaker.name}>
                <SelectLabel>{speaker.name}</SelectLabel>
                {speaker.styles.map((style) => (
                  <SelectItem key={style.id} value={String(style.id)}>
                    {speaker.name}-{style.name}
                  </SelectItem>
                ))}
              </SelectGroup>
            ))}
          </SelectContent>
        </Select>
      </label>

      <div className="my-[var(--hud-space-xs)] h-px bg-[oklch(0.72_0.14_192/0.1)]" />
      <div className="relative z-[7] text-[var(--hud-font-size-sm)] font-medium tracking-[0.06em] text-[oklch(0.72_0.14_192/0.5)]">
        Voice Parameters
      </div>

      {PARAMS.map((param) => (
        <label
          key={param.key}
          className="flex flex-col gap-[var(--hud-space-sm)] text-[var(--hud-font-size-sm)] uppercase tracking-[0.1em] text-[oklch(0.72_0.14_192/0.7)]"
        >
          {param.label}
          <div className="flex flex-row items-center gap-[var(--hud-space-lg)]">
            <input
              type="range"
              className="settings-slider"
              min={param.min}
              max={param.max}
              step={param.step}
              value={settings[param.key] as number}
              onChange={(e) =>
                onSettingsChange({
                  ...settings,
                  [param.key]: parseFloat(e.target.value),
                })
              }
            />
            <span className="min-w-[3.5em] text-right font-mono text-[var(--hud-font-size-sm)] text-[oklch(0.72_0.14_192)]">
              {(settings[param.key] as number).toFixed(2)}
            </span>
          </div>
          <div className="mt-0.5 text-[var(--hud-font-size-xs)] text-[oklch(0.72_0.14_192/0.4)]">
            {param.desc}
          </div>
        </label>
      ))}

      <div className="my-[var(--hud-space-xs)] h-px bg-[oklch(0.72_0.14_192/0.1)]" />
      <SpeechTest speaking={speaking} disabled={disabled} onSpeak={onSpeak} />
    </>
  );
}

function SpeechTest({
  speaking,
  disabled,
  onSpeak,
}: {
  speaking: boolean;
  disabled: boolean;
  onSpeak: (text: string) => void;
}) {
  const [text, setText] = useState('');

  return (
    <div className="relative z-[7] flex flex-col gap-[var(--hud-space-md)]">
      <div className="relative z-[7] text-[var(--hud-font-size-sm)] font-medium tracking-[0.06em] text-[oklch(0.72_0.14_192/0.5)]">
        Speech Test
      </div>
      <Textarea
        className="resize-none"
        rows={3}
        placeholder="Enter text to test..."
        value={text}
        onChange={(e) => setText(e.target.value)}
        disabled={disabled || speaking}
      />
      <div className="flex justify-end">
        <button
          type="button"
          className="cursor-pointer rounded-md border border-[oklch(0.72_0.14_192/0.3)] bg-[oklch(0.72_0.14_192/0.15)] px-[var(--hud-space-2xl)] py-[var(--hud-space-md)] font-[inherit] text-[var(--hud-font-size-sm)] uppercase tracking-[0.08em] text-[oklch(0.72_0.14_192)] transition-[background,border-color,box-shadow] duration-[var(--hud-duration-content)] ease-out enabled:hover:border-[oklch(0.72_0.14_192/0.5)] enabled:hover:bg-[oklch(0.72_0.14_192/0.25)] enabled:hover:[box-shadow:0_0_12px_oklch(0.72_0.14_192/0.2)] enabled:active:scale-[0.97] disabled:cursor-not-allowed disabled:opacity-50"
          disabled={disabled || speaking || text.trim().length === 0}
          onClick={() => onSpeak(text.trim())}
        >
          {speaking ? 'Speaking...' : '▶ Speak'}
        </button>
      </div>
    </div>
  );
}

function Footer({
  onClose,
  onReset,
  onSave,
  saving,
  saved,
  disabled,
}: {
  onClose: () => void;
  onReset: () => void;
  onSave: () => void;
  saving: boolean;
  saved: boolean;
  disabled: boolean;
}) {
  const closeBtn =
    'cursor-pointer rounded-md border border-[oklch(0.4_0.02_250/0.25)] bg-transparent px-[var(--hud-space-2xl)] py-[var(--hud-space-md)] font-[inherit] text-[var(--hud-font-size-sm)] uppercase tracking-[0.08em] text-[oklch(0.75_0.04_250/0.75)] transition-[color,border-color,text-shadow] duration-[var(--hud-duration-content)] ease-out hover:border-[oklch(0.55_0.04_250/0.35)] hover:text-[oklch(0.88_0.08_250/0.9)] hover:[text-shadow:0_0_10px_oklch(0.72_0.14_192/0.2)] active:scale-[0.97]';

  return (
    <div className="relative z-[7] flex justify-end gap-[var(--hud-space-md)] pt-[var(--hud-space-md)]">
      <button type="button" className={closeBtn} onClick={onClose}>
        Close
      </button>
      <button type="button" className={closeBtn} onClick={onReset}>
        Reset
      </button>
      <button
        type="button"
        className={cn(
          'cursor-pointer rounded-md border px-[var(--hud-space-2xl)] py-[var(--hud-space-md)] font-[inherit] text-[var(--hud-font-size-sm)] uppercase tracking-[0.08em] transition-[background,border-color,box-shadow,color] duration-[var(--hud-duration-content)] ease-out active:scale-[0.97] disabled:cursor-not-allowed disabled:opacity-50',
          saved
            ? 'border-[oklch(0.65_0.18_145/0.4)] bg-[oklch(0.65_0.18_145/0.15)] text-[oklch(0.65_0.18_145)] [box-shadow:0_0_12px_oklch(0.65_0.18_145/0.15)] hover:border-[oklch(0.65_0.18_145/0.5)] hover:bg-[oklch(0.65_0.18_145/0.2)] hover:[box-shadow:0_0_16px_oklch(0.65_0.18_145/0.25)]'
            : 'border-[oklch(0.72_0.14_192/0.3)] bg-[oklch(0.72_0.14_192/0.15)] text-[oklch(0.72_0.14_192)] hover:border-[oklch(0.72_0.14_192/0.5)] hover:bg-[oklch(0.72_0.14_192/0.25)] hover:[box-shadow:0_0_12px_oklch(0.72_0.14_192/0.2)]',
        )}
        onClick={onSave}
        disabled={saving || disabled}
      >
        {saving ? 'Saving...' : saved ? 'Saved!' : 'Save'}
      </button>
    </div>
  );
}
