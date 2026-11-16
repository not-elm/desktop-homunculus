import { mods, Webview } from '@hmcs/sdk';
import { useCallback, useRef, useState } from 'react';
import type { MenuItemData } from './useMenuData';

export function useMenuActions(personaId: string | null) {
  const [closing, setClosing] = useState(false);

  const handleClose = useCallback(async () => {
    if (closing) return;
    setClosing(true);
    setTimeout(async () => {
      try {
        await Webview.current()?.close();
      } catch (err) {
        console.error('Failed to close the menu:', err);
      }
    }, 180);
  }, [closing]);

  const selectedRef = useRef(false);

  const handleSelect = useCallback(
    (item: MenuItemData) => {
      if (selectedRef.current) return;
      selectedRef.current = true;
      handleClose();
      if (item.command && personaId != null) {
        mods
          .executeCommand({
            command: item.command,
            stdin: JSON.stringify({ linkedPersona: personaId }),
          })
          .catch((err) => {
            console.error('Command execution failed:', err);
          });
      }
    },
    [personaId, handleClose],
  );

  return { closing, handleClose, handleSelect };
}
