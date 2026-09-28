<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { api, type AppInfo } from './api';
  import { toast, guard } from './toast.svelte';
  import Modal from './Modal.svelte';
  import AppIcon from './AppIcon.svelte';

  let { onclose, onadded }: { onclose: () => void; onadded: () => void } = $props();

  let tab = $state<'instaladas' | 'abiertas' | 'exe'>('instaladas');
  let loading = $state(false);
  let installed = $state<AppInfo[]>([]);
  let running = $state<AppInfo[]>([]);
  let query = $state('');
  let picked = $state<AppInfo | null>(null);

  // Config de la app elegida
  let authKind = $state<'own' | 'master'>('own');
  let appSecret = $state('');
  let trust = $state(0);
  let busy = $state(false);

  const source = $derived(tab === 'instaladas' ? installed : running);
  const filtered = $derived(
    source.filter((a) => a.display.toLowerCase().includes(query.toLowerCase()))
  );

  async function loadInstalled() {
    if (installed.length) return;
    loading = true;
    try {
      installed = await api.listInstalled();
    } catch (e) {
      toast(String(e), 'error');
    }
    loading = false;
  }
  async function loadRunning() {
    loading = true;
    try {
      running = await api.listRunning();
    } catch (e) {
      toast(String(e), 'error');
    }
    loading = false;
  }

  function switchTab(t: typeof tab) {
    tab = t;
    query = '';
    if (t === 'instaladas') loadInstalled();
    else if (t === 'abiertas') loadRunning();
  }

  async function pickExe() {
    const file = await open({ filters: [{ name: 'Programas', extensions: ['exe'] }] });
    if (typeof file === 'string') {
      picked = await api.appFromPath(file);
    }
  }

  async function confirmAdd() {
    if (!picked) return;
    if (authKind === 'own' && appSecret.trim().length < 4) {
      toast('La contraseña de la app debe tener al menos 4 caracteres', 'error');
      return;
    }
    busy = true;
    const ok = await guard(
      () =>
        api.addApp({
          display: picked!.display,
          exe: picked!.exe,
          path: picked!.path,
          icon: picked!.icon,
          authKind,
          appSecret,
          trustMinutes: trust
        }),
      `${picked.display} protegida`
    );
    busy = false;
    if (ok) onadded();
  }

  loadInstalled();
</script>

<Modal title={picked ? 'Configurar protección' : 'Añadir app'} onclose={onclose} wide>
  {#if !picked}
    <div class="tabs">
      <button class:on={tab === 'instaladas'} onclick={() => switchTab('instaladas')}>Instaladas</button>
      <button class:on={tab === 'abiertas'} onclick={() => switchTab('abiertas')}>Abiertas ahora</button>
      <button class:on={tab === 'exe'} onclick={() => switchTab('exe')}>Elegir .exe</button>
    </div>

    {#if tab === 'exe'}
      <div class="exe">
        <div class="exe-ico">📁</div>
        <p>Elige cualquier ejecutable: juegos portables, apps sueltas…</p>
        <button class="btn" onclick={pickExe}>Buscar .exe</button>
      </div>
    {:else}
      <input class="field" placeholder="🔍 Buscar…" bind:value={query} style="margin-bottom:12px" />
      {#if loading}
        <div class="empty">Cargando…</div>
      {:else if filtered.length === 0}
        <div class="empty">Nada por aquí</div>
      {:else}
        <div class="list">
          {#each filtered as app (app.exe + app.display)}
            <button class="item" onclick={() => (picked = app)}>
              <AppIcon icon={app.icon} name={app.display} />
              <div class="meta">
                <span class="t">{app.display}</span>
                <span class="s">{app.exe}</span>
              </div>
              <span class="add">Elegir</span>
            </button>
          {/each}
        </div>
      {/if}
    {/if}
  {:else}
    <div class="chosen">
      <AppIcon icon={picked.icon} name={picked.display} big />
      <div>
        <div class="ct">{picked.display}</div>
        <div class="cs">{picked.exe}</div>
      </div>
      <button class="btn ghost sm" onclick={() => (picked = null)} style="margin-left:auto">Cambiar</button>
    </div>

    <div class="label" style="margin-bottom:8px">Contraseña de esta app</div>
    <div class="seg">
      <button class:on={authKind === 'own'} onclick={() => (authKind = 'own')}>Propia</button>
      <button class:on={authKind === 'master'} onclick={() => (authKind = 'master')}>Usar la maestra</button>
    </div>
    {#if authKind === 'own'}
      <input
        class="field"
        type="password"
        placeholder="Contraseña para abrir esta app"
        bind:value={appSecret}
        style="margin-top:10px"
      />
    {/if}

    <div class="label" style="margin:16px 0 8px">Tiempo de confianza</div>
    <p class="hint">Tras desbloquear, no volver a pedir la contraseña durante:</p>
    <div class="chips">
      {#each [ [0,'Sin confianza'], [5,'5 min'], [15,'15 min'], [60,'1 hora'] ] as [v, lbl]}
        <button class:on={trust === v} onclick={() => (trust = v as number)}>{lbl}</button>
      {/each}
    </div>

    <button class="btn" style="width:100%;margin-top:20px" disabled={busy} onclick={confirmAdd}>
      Proteger {picked.display}
    </button>
  {/if}
</Modal>

<style>
  .tabs {
    display: flex;
    gap: 4px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 4px;
    margin-bottom: 14px;
  }
  .tabs button {
    flex: 1;
    padding: 9px;
    border: 0;
    border-radius: 9px;
    background: transparent;
    color: var(--muted);
    font-weight: 600;
    font-size: 13px;
  }
  .tabs button.on {
    background: var(--surface3);
    color: var(--text);
  }
  .list {
    max-height: 320px;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 9px 11px;
    border-radius: 12px;
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--text);
    text-align: left;
    transition: background 0.12s ease;
  }
  .item:hover {
    background: var(--surface3);
  }
  .meta {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .meta .t {
    font-size: 14px;
    font-weight: 600;
  }
  .meta .s {
    font-size: 11px;
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .add {
    margin-left: auto;
    font-size: 12px;
    font-weight: 700;
    color: var(--accent);
  }
  .empty {
    text-align: center;
    color: var(--muted);
    padding: 40px;
    font-size: 14px;
  }
  .exe {
    text-align: center;
    padding: 30px 20px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
  }
  .exe-ico {
    font-size: 44px;
  }
  .exe p {
    color: var(--muted);
    font-size: 13px;
  }
  .chosen {
    display: flex;
    align-items: center;
    gap: 13px;
    padding: 13px;
    border-radius: 14px;
    background: var(--surface);
    border: 1px solid var(--border);
    margin-bottom: 18px;
  }
  .ct {
    font-size: 15px;
    font-weight: 700;
  }
  .cs {
    font-size: 12px;
    color: var(--muted);
  }
  .seg {
    display: flex;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 4px;
    gap: 4px;
  }
  .seg button {
    flex: 1;
    padding: 9px;
    border: 0;
    border-radius: 9px;
    background: transparent;
    color: var(--muted);
    font-weight: 600;
    font-size: 13px;
  }
  .seg button.on {
    background: var(--surface3);
    color: var(--text);
  }
  .hint {
    font-size: 12px;
    color: var(--muted);
    margin-bottom: 10px;
  }
  .chips {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .chips button {
    padding: 8px 14px;
    border-radius: 10px;
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--muted);
    font-weight: 600;
    font-size: 13px;
  }
  .chips button.on {
    background: var(--accent-soft);
    border-color: var(--accent);
    color: var(--accent);
  }
</style>
