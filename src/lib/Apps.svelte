<script lang="ts">
  import { fly } from 'svelte/transition';
  import { flip } from 'svelte/animate';
  import { api, type AppView } from './api';
  import { guard, toast } from './toast.svelte';
  import { askMaster } from './prompt.svelte';
  import AppIcon from './AppIcon.svelte';
  import AddApp from './AddApp.svelte';
  import Modal from './Modal.svelte';

  let { apps, refresh }: { apps: AppView[]; refresh: () => void } = $props();

  let query = $state('');
  let adding = $state(false);
  let menuFor = $state<string | null>(null);
  let editing = $state<AppView | null>(null);

  // Estado de edición
  let eKind = $state<'own' | 'master'>('own');
  let eSecret = $state('');
  let eTrust = $state(0);

  const filtered = $derived(apps.filter((a) => a.display.toLowerCase().includes(query.toLowerCase())));

  function trustLabel(m: number) {
    if (m === 0) return 'Sin confianza';
    if (m === 60) return 'Confianza 1 h';
    return `Confianza ${m} min`;
  }

  async function toggle(app: AppView) {
    if (app.enabled) {
      const m = await askMaster(`Vas a desactivar la protección de ${app.display}.`);
      if (m === null) return;
      await guard(() => api.setAppEnabled(app.exe, false, m), `${app.display} en pausa`);
    } else {
      await guard(() => api.setAppEnabled(app.exe, true, ''), `${app.display} protegida`);
    }
    refresh();
  }

  async function remove(app: AppView) {
    menuFor = null;
    const m = await askMaster(`Vas a quitar la protección de ${app.display}.`);
    if (m === null) return;
    await guard(() => api.removeApp(app.exe, m), `${app.display} eliminada`);
    refresh();
  }

  function openEdit(app: AppView) {
    menuFor = null;
    editing = app;
    eKind = app.auth_kind as 'own' | 'master';
    eSecret = '';
    eTrust = app.trust_minutes;
  }

  async function saveEdit() {
    if (!editing) return;
    if (eKind === 'own' && eSecret && eSecret.trim().length < 4) {
      toast('La contraseña debe tener al menos 4 caracteres', 'error');
      return;
    }
    // Cambiar contraseña solo si eligió maestra o escribió una nueva propia.
    if (eKind === 'master' || eSecret) {
      const ok = await guard(() => api.changeAppPassword(editing!.exe, eKind, eSecret));
      if (!ok) return;
    }
    if (eTrust !== editing.trust_minutes) {
      await guard(() => api.setAppTrust(editing!.exe, eTrust));
    }
    toast('Cambios guardados', 'ok');
    editing = null;
    refresh();
  }
</script>

<div class="head">
  <div>
    <h2>Apps protegidas</h2>
    <p class="sub">{apps.length} {apps.length === 1 ? 'aplicación' : 'aplicaciones'} bajo llave</p>
  </div>
  <button class="btn" onclick={() => (adding = true)}>+ Añadir app</button>
</div>

{#if apps.length > 4}
  <input class="field" placeholder="🔍 Buscar…" bind:value={query} style="margin-bottom:14px" />
{/if}

{#if apps.length === 0}
  <div class="empty" in:fly={{ y: 10, duration: 250 }}>
    <div class="e-ico">🛡️</div>
    <h3>Aún no proteges ninguna app</h3>
    <p>Añade una y Asegurao pedirá la contraseña cada vez que se abra.</p>
    <button class="btn" onclick={() => (adding = true)}>+ Añadir la primera</button>
  </div>
{:else}
  <div class="list">
    {#each filtered as app (app.exe)}
      <div class="row" class:off={!app.enabled} animate:flip={{ duration: 200 }} in:fly={{ y: 8, duration: 200 }}>
        <AppIcon icon={app.icon} name={app.display} />
        <div class="meta">
          <span class="t">{app.display}</span>
          <span class="s">{app.path || app.exe}</span>
        </div>
        <span class="tag">{app.auth_kind === 'master' ? 'Maestra' : 'Propia'} · {trustLabel(app.trust_minutes)}</span>
        <button
          class="switch"
          class:on={app.enabled}
          onclick={() => toggle(app)}
          aria-label="Activar o pausar"
        ><span></span></button>
        <div class="menu-wrap">
          <button class="dots" onclick={() => (menuFor = menuFor === app.exe ? null : app.exe)} aria-label="Más">⋯</button>
          {#if menuFor === app.exe}
            <div class="menu" in:fly={{ y: -6, duration: 140 }}>
              <button onclick={() => openEdit(app)}>Editar</button>
              <button class="danger" onclick={() => remove(app)}>Quitar protección</button>
            </div>
          {/if}
        </div>
      </div>
    {/each}
  </div>
{/if}

{#if adding}
  <AddApp onclose={() => (adding = false)} onadded={() => { adding = false; refresh(); }} />
{/if}

{#if editing}
  <Modal title={`Editar · ${editing.display}`} onclose={() => (editing = null)}>
    <div class="label" style="margin-bottom:8px">Contraseña de esta app</div>
    <div class="seg">
      <button class:on={eKind === 'own'} onclick={() => (eKind = 'own')}>Propia</button>
      <button class:on={eKind === 'master'} onclick={() => (eKind = 'master')}>Usar la maestra</button>
    </div>
    {#if eKind === 'own'}
      <input class="field" type="password" placeholder="Nueva contraseña (deja vacío para no cambiarla)" bind:value={eSecret} style="margin-top:10px" />
    {/if}

    <div class="label" style="margin:16px 0 10px">Tiempo de confianza</div>
    <div class="chips">
      {#each [ [0,'Sin confianza'], [5,'5 min'], [15,'15 min'], [60,'1 hora'] ] as [v, lbl]}
        <button class:on={eTrust === v} onclick={() => (eTrust = v as number)}>{lbl}</button>
      {/each}
    </div>
    <button class="btn" style="width:100%;margin-top:20px" onclick={saveEdit}>Guardar</button>
  </Modal>
{/if}

<style>
  .head {
    display: flex;
    align-items: center;
    margin-bottom: 18px;
  }
  .head h2 {
    font-size: 22px;
    font-weight: 800;
  }
  .sub {
    color: var(--muted);
    font-size: 13px;
    margin-top: 2px;
  }
  .head .btn {
    margin-left: auto;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 13px;
    padding: 11px 14px;
    border-radius: 14px;
    background: var(--surface);
    border: 1px solid var(--border);
    transition: opacity 0.2s ease;
  }
  .row.off {
    opacity: 0.5;
  }
  .meta {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: 1;
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
    max-width: 340px;
  }
  .tag {
    font-size: 11px;
    color: var(--muted);
    white-space: nowrap;
  }
  .switch {
    width: 40px;
    height: 23px;
    border-radius: 12px;
    background: var(--surface3);
    border: 1px solid var(--border2);
    position: relative;
    transition: background 0.2s ease;
    flex: none;
  }
  .switch span {
    position: absolute;
    left: 2px;
    top: 2px;
    width: 17px;
    height: 17px;
    border-radius: 50%;
    background: var(--muted);
    transition: transform 0.2s ease, background 0.2s ease;
  }
  .switch.on {
    background: var(--accent);
    border-color: var(--accent);
  }
  .switch.on span {
    transform: translateX(17px);
    background: var(--accent-ink);
  }
  .menu-wrap {
    position: relative;
  }
  .dots {
    background: var(--surface2);
    border: 1px solid var(--border);
    color: var(--muted);
    width: 30px;
    height: 30px;
    border-radius: 9px;
    font-size: 17px;
    line-height: 1;
  }
  .dots:hover {
    color: var(--text);
    background: var(--surface3);
  }
  .menu {
    position: absolute;
    right: 0;
    top: 36px;
    background: var(--app-bg);
    background-attachment: fixed;
    border: 1px solid var(--border2);
    border-radius: 11px;
    box-shadow: var(--shadow);
    overflow: hidden;
    z-index: 30;
    min-width: 160px;
  }
  .menu button {
    display: block;
    width: 100%;
    text-align: left;
    padding: 10px 14px;
    background: none;
    border: 0;
    color: var(--text);
    font-size: 13px;
    font-weight: 600;
  }
  .menu button:hover {
    background: var(--surface2);
  }
  .menu button.danger {
    color: var(--danger);
  }
  .empty {
    text-align: center;
    padding: 60px 20px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
  }
  .e-ico {
    font-size: 50px;
  }
  .empty h3 {
    font-size: 18px;
  }
  .empty p {
    color: var(--muted);
    font-size: 14px;
    margin-bottom: 8px;
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
