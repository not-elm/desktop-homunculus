import { Persona } from '@hmcs/sdk';
import {
  cn,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@hmcs/ui';
import { BehaviorSection } from '@persona/shared/components/BehaviorSection';
import { PersonaDetailBody } from '@persona/shared/components/PersonaDetailBody';
import { usePersonaDetail } from '@persona/shared/hooks/usePersonaDetail';
import { useThumbnailImport } from '@persona/shared/hooks/useThumbnailImport';
import { useMemo, useState } from 'react';

type Tab = 'persona' | 'appearance';

interface DetailViewProps {
  personaId: string;
  onDirtyChange: (dirty: boolean) => void;
  onSaved: () => void;
  onDelete: () => Promise<void>;
}

export default function DetailView({
  personaId,
  onDirtyChange,
  onSaved,
  onDelete,
}: DetailViewProps) {
  const callbacks = useMemo(() => ({ onDirtyChange, onSaved }), [onDirtyChange, onSaved]);
  const {
    snapshot,
    formValues,
    vrmAssetId,
    thumbnail,
    setThumbnail,
    saving,
    saved,
    setFormValues,
    setVrmAssetId,
    save,
    toggleSpawn,
    toggleAutoSpawn,
    behaviorProcess,
    behaviorAnimations,
    setBehaviorProcess,
    setBehaviorAnimations,
  } = usePersonaDetail(personaId, callbacks);

  const persona = useMemo(() => new Persona(personaId), [personaId]);
  const { importThumbnail } = useThumbnailImport();
  const [tab, setTab] = useState<Tab>('persona');
  const [deleteOpen, setDeleteOpen] = useState(false);

  if (!snapshot || !formValues) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <div className="text-xs uppercase tracking-[0.12em] text-primary/50">Loading...</div>
      </div>
    );
  }

  const autoSpawn = snapshot.metadata?.['auto-spawn'] === true;

  async function handleThumbnailChange() {
    const assetId = await importThumbnail(personaId);
    if (assetId) {
      setThumbnail(assetId);
    }
  }

  async function handleDelete() {
    try {
      await onDelete();
    } catch (e) {
      console.error('Failed to delete persona:', e);
    }
  }

  return (
    <div className="flex flex-1 min-w-0 flex-col">
      <DetailHeader
        name={snapshot.name ?? ''}
        personaId={personaId}
        isSpawned={snapshot.spawned}
        onSpawnToggle={toggleSpawn}
        onSave={save}
        saving={saving}
        saved={saved}
      />

      <Tabs
        value={tab}
        onValueChange={(v) => setTab(v as Tab)}
        className="flex flex-1 min-h-0 flex-col"
      >
        <TabsList className="mx-5 mt-3 self-start">
          <TabsTrigger value="persona">Persona</TabsTrigger>
          <TabsTrigger value="appearance">Appearance</TabsTrigger>
        </TabsList>

        <TabsContent value="persona" className="no-scrollbar flex-1 overflow-y-auto p-5">
          <PersonaDetailBody
            personaId={personaId}
            thumbnailUrl={persona.thumbnailUrl(thumbnail)}
            onThumbnailChange={handleThumbnailChange}
            vrmAssetId={vrmAssetId}
            onVrmChange={setVrmAssetId}
            autoSpawn={autoSpawn}
            onAutoSpawnToggle={toggleAutoSpawn}
            formValues={formValues}
            onFormChange={setFormValues}
          />
        </TabsContent>

        <TabsContent value="appearance" className="no-scrollbar flex-1 overflow-y-auto p-5">
          <BehaviorSection
            process={behaviorProcess}
            animations={behaviorAnimations}
            onProcessChange={setBehaviorProcess}
            onAnimationsChange={setBehaviorAnimations}
          />
        </TabsContent>
      </Tabs>

      <DeleteSection onDelete={() => setDeleteOpen(true)} />

      <DeleteConfirmDialog
        open={deleteOpen}
        onOpenChange={setDeleteOpen}
        onConfirm={handleDelete}
      />
    </div>
  );
}

function DetailHeader({
  name,
  personaId,
  isSpawned,
  onSpawnToggle,
  onSave,
  saving,
  saved,
}: {
  name: string;
  personaId: string;
  isSpawned: boolean;
  onSpawnToggle: () => void;
  onSave: () => void;
  saving: boolean;
  saved: boolean;
}) {
  return (
    <div className="flex shrink-0 items-center justify-between gap-3 border-b border-primary/12 px-5 py-3">
      <div className="flex min-w-0 flex-col">
        <span className="truncate text-sm font-semibold text-foreground">{name}</span>
        <span className="truncate font-mono text-[10px] tracking-[0.04em] text-hud-text-subdued">
          {personaId}
        </span>
      </div>
      <div className="flex shrink-0 items-center gap-2">
        <button
          type="button"
          onClick={onSpawnToggle}
          disabled={saving}
          className={cn(
            'cursor-pointer rounded-md border px-4 py-1.5 text-xs uppercase tracking-[0.08em] transition-colors duration-200 disabled:cursor-not-allowed disabled:opacity-50',
            isSpawned
              ? 'border-holo-rose/30 bg-holo-rose/15 text-holo-rose hover:border-holo-rose/50 hover:bg-holo-rose/25'
              : 'border-primary/30 bg-primary/15 text-primary hover:border-primary/50 hover:bg-primary/25',
          )}
        >
          {isSpawned ? 'Despawn' : 'Spawn'}
        </button>
        <button
          type="button"
          onClick={onSave}
          disabled={saving}
          className={cn(
            'cursor-pointer rounded-md border px-5 py-1.5 text-xs uppercase tracking-[0.08em] transition-[background,border-color,box-shadow,color] duration-200 active:scale-[0.97] disabled:cursor-not-allowed disabled:opacity-50',
            saved
              ? 'border-success/40 bg-success/15 text-success hover:border-success/50 hover:bg-success/20 hover:shadow-holo-sm'
              : 'border-primary/30 bg-primary/15 text-primary hover:border-primary/50 hover:bg-primary/25 hover:shadow-holo-sm',
          )}
        >
          {saving ? 'Saving...' : saved ? 'Saved!' : 'Save'}
        </button>
      </div>
    </div>
  );
}

function DeleteSection({ onDelete }: { onDelete: () => void }) {
  return (
    <div className="flex shrink-0 justify-end border-t border-primary/12 px-5 py-3">
      <button
        type="button"
        onClick={onDelete}
        className="cursor-pointer rounded-md border border-destructive/30 bg-destructive/15 px-5 py-2 text-xs uppercase tracking-[0.08em] text-destructive transition-colors duration-200 hover:border-destructive/50 hover:bg-destructive/25"
      >
        Delete Persona
      </button>
    </div>
  );
}

function DeleteConfirmDialog({
  open,
  onOpenChange,
  onConfirm,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
}) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent showCloseButton={false}>
        <DialogHeader>
          <DialogTitle>Delete Persona</DialogTitle>
          <DialogDescription>This action cannot be undone. Are you sure?</DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <button
            type="button"
            className="cursor-pointer rounded-md border border-muted-foreground/25 bg-transparent px-5 py-2 text-xs uppercase tracking-[0.08em] text-muted-foreground transition-colors duration-200 hover:border-muted-foreground/45 hover:text-foreground"
            onClick={() => onOpenChange(false)}
          >
            Cancel
          </button>
          <button
            type="button"
            className="cursor-pointer rounded-md border border-destructive/40 bg-destructive/15 px-5 py-2 text-xs uppercase tracking-[0.08em] text-destructive transition-colors duration-200 hover:border-destructive/55 hover:bg-destructive/25"
            onClick={onConfirm}
          >
            Delete
          </button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
