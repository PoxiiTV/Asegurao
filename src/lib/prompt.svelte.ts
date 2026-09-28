// Solicita la contraseña maestra para acciones que debilitan la protección.
// (El backend, además, dispara la confirmación de Windows / UAC.)

interface MasterReq {
  open: boolean;
  note: string;
  resolve?: (v: string | null) => void;
}

export const masterReq = $state<MasterReq>({ open: false, note: '' });

export function askMaster(note = ''): Promise<string | null> {
  return new Promise((resolve) => {
    masterReq.open = true;
    masterReq.note = note;
    masterReq.resolve = resolve;
  });
}

export function resolveMaster(value: string | null) {
  masterReq.resolve?.(value);
  masterReq.open = false;
  masterReq.note = '';
  masterReq.resolve = undefined;
}
