<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { api, applyTheme, type Settings, type AuthKind } from './api';
  import { guard, toast } from './toast.svelte';
  import Modal from './Modal.svelte';

  let {
    settings,
    authKind,
    refresh
  }: { settings: Settings; authKind: string; refresh: () => void } = $props();

  const VERSION = '1.0.0';
  const REPO = 'Poxi/Asegurao'; // ajusta a tu repositorio real

  let s = $state<Settings>({ ...settings, intruder: { ...settings.intruder } });
  let changingPass = $state(false);
  let checking = $state(false);

  // Cambio de contraseña maestra
  let cCurrent = $state('');
  let cKind = $state<AuthKind>(authKind as AuthKind);
  let cNew = $state('');

  const themes = [
    { id: 'grafito', name: 'Grafito', c1: '#2a2c31', c2: '#ffb020' },
    { id: 'boveda', name: 'Bóveda', c1: '#05070d', c2: '#2de2c4' },
    { id: 'aurora', name: 'Aurora', c1: '#3a0ca3', c2: '#ff4d97' }
  ];

  async function pickTheme(id: string) {
    s.theme = id;
    applyTheme(id);
    await guard(() => api.setTheme(id));
  }

  async function persist() {
    await guard(() => api.saveSettings({ ...s, intruder: { ...s.intruder } }), 'Ajustes guardados');
    refresh();
  }

  async function toggleAutostart(on: boolean) {
    s.start_with_windows = on;
    await guard(() => api.setAutostart(on));
    await api.saveSettings({ ...s, intruder: { ...s.intruder } });
  }

  async function changePassword() {
    if (cNew.length < (cKind === 'pin' ? 6 : 8)) {
      toast(`La nueva ${cKind === 'pin' ? 'clave' : 'contraseña'} es demasiado corta`, 'error');
      return;
    }
    const ok = await guard(
      () => api.changeMaster(cCurrent, cKind, cNew),
      'Contraseña maestra actualizada'
    );
    if (ok) {
      changingPass = false;
      cCurrent = cNew = '';
      refresh();
    }
  }

  async function checkUpdate() {
    checking = true;
    try {
      const r = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`);
      if (!r.ok) throw new Error('sin conexión');
      const j = await r.json();
      const latest = (j.tag_name || '').replace(/^v/, '');
      if (latest && latest > VERSION) {
        toast(`Hay una versión nueva (${latest})`, 'ok');
        if (j.html_url) openUrl(j.html_url);
      } else {
        toast('Ya tienes la última versión', 'ok');
      }
    } catch {
      toast('No se pudo comprobar las actualizaciones', 'error');
    }
    checking = false;
  }
</script>

<h2>Ajustes</h2>

<section>
  <div class="label">Apariencia</div>
  <div class="themes">
    {#each themes as t}
      <button class="theme" class:on={s.theme === t.id} onclick={() => pickTheme(t.id)}>
        <div class="swatch" style={`background:linear-gradient(135deg, ${t.c1}, ${t.c1});`}>
          <span class="acc" style={`background:${t.c2}`}></span>
        </div>
        {t.name}
      </button>
    {/each}
  </div>
</section>

<section>
  <div class="label">Seguridad</div>
  <div class="rows">
    <div class="opt">
      <div>
        <b>Contraseña maestra</b>
        <p>La llave para entrar y autorizar cambios.</p>
      </div>
      <button class="btn ghost sm" onclick={() => (changingPass = true)}>Cambiar</button>
    </div>

    <div class="opt">
      <div>
        <b>Bloqueo automático</b>
        <p>Cierra la sesión de Asegurao tras un tiempo sin uso.</p>
      </div>
      <select class="sel" bind:value={s.autolock_minutes} onchange={persist}>
        <option value={0}>Nunca</option>
        <option value={1}>1 min</option>
        <option value={5}>5 min</option>
        <option value={15}>15 min</option>
        <option value={30}>30 min</option>
      </select>
    </div>

    <div class="opt">
      <div>
        <b>Nivel Fortaleza</b>
        <p>Windows pedirá tu contraseña para desactivar la protección o desinstalar.</p>
      </div>
      <button
        class="switch"
        class:on={s.level === 'fortaleza'}
        onclick={() => { s.level = s.level === 'fortaleza' ? 'base' : 'fortaleza'; persist(); }}
        aria-label="Nivel fortaleza"
      ><span></span></button>
    </div>

    <div class="opt">
      <div>
        <b>Foto del intruso</b>
        <p>Hace una foto con la webcam tras varios fallos seguidos.</p>
      </div>
      <button
        class="switch"
        class:on={s.intruder.enabled}
        onclick={() => { s.intruder.enabled = !s.intruder.enabled; persist(); }}
        aria-label="Foto del intruso"
      ><span></span></button>
    </div>
    {#if s.intruder.enabled}
      <div class="opt sub-opt">
        <div><b>Tras cuántos fallos</b></div>
        <select class="sel" bind:value={s.intruder.after_fails} onchange={persist}>
          <option value={1}>1 fallo</option>
          <option value={3}>3 fallos</option>
          <option value={5}>5 fallos</option>
        </select>
      </div>
    {/if}

    <div class="opt">
      <div>
        <b>Iniciar con Windows</b>
        <p>Asegurao arranca al encender el ordenador.</p>
      </div>
      <button
        class="switch"
        class:on={s.start_with_windows}
        onclick={() => toggleAutostart(!s.start_with_windows)}
        aria-label="Iniciar con Windows"
      ><span></span></button>
    </div>
  </div>
</section>

<section>
  <div class="label">Acerca de</div>
  <div class="about">
    <div class="a-logo">🛡️</div>
    <div>
      <b>Asegurao <span class="ver">v{VERSION}</span></b>
      <p>Protege tus aplicaciones con contraseña.</p>
    </div>
    <button class="btn ghost sm" disabled={checking} onclick={checkUpdate} style="margin-left:auto">
      {checking ? 'Comprobando…' : 'Buscar actualizaciones'}
    </button>
  </div>
</section>

{#if changingPass}
  <Modal title="Cambiar contraseña maestra" onclose={() => (changingPass = false)}>
    <p class="note">Windows te pedirá también tu contraseña para confirmar.</p>
    <input class="field" type="password" placeholder="Contraseña actual" bind:value={cCurrent} style="margin-bottom:10px" />
    <div class="seg">
      <button class:on={cKind === 'pin'} onclick={() => (cKind = 'pin')}>PIN</button>
      <button class:on={cKind === 'password'} onclick={() => (cKind = 'password')}>Contraseña</button>
    </div>
    <input class="field" type="password" placeholder={cKind === 'pin' ? 'Nuevo PIN (mín. 6)' : 'Nueva contraseña (mín. 8)'} bind:value={cNew} style="margin-top:10px" />
    <button class="btn" style="width:100%;margin-top:16px" onclick={changePassword}>Cambiar</button>
  </Modal>
{/if}

<style>
  h2 {
    font-size: 22px;
    font-weight: 800;
    margin-bottom: 20px;
  }
  section {
    margin-bottom: 26px;
  }
  .label {
    margin-bottom: 12px;
  }
  .themes {
    display: flex;
    gap: 12px;
  }
  .theme {
    background: none;
    border: 0;
    color: var(--muted);
    font-size: 13px;
    font-weight: 600;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }
  .swatch {
    width: 84px;
    height: 56px;
    border-radius: 12px;
    border: 2px solid var(--border);
    position: relative;
    overflow: hidden;
    transition: border-color 0.15s ease;
  }
  .theme.on {
    color: var(--text);
  }
  .theme.on .swatch {
    border-color: var(--accent);
  }
  .acc {
    position: absolute;
    right: 10px;
    bottom: 10px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
  }
  .rows {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .opt {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 14px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 13px;
  }
  .opt + .opt {
    margin-top: 8px;
  }
  .sub-opt {
    margin-top: 8px;
    margin-left: 20px;
    background: var(--surface2);
  }
  .opt b {
    font-size: 14px;
  }
  .opt p {
    font-size: 12px;
    color: var(--muted);
    margin-top: 3px;
    max-width: 420px;
    line-height: 1.4;
  }
  .opt > div:first-child {
    flex: 1;
  }
  .sel {
    background: #26282d;
    border: 1px solid var(--border2);
    color: var(--text);
    border-radius: 10px;
    padding: 8px 12px;
    font-size: 13px;
    font-family: inherit;
    font-weight: 600;
    outline: none;
    color-scheme: dark;
    cursor: pointer;
  }
  .sel option {
    background: #26282d;
    color: var(--text);
  }
  .switch {
    width: 44px;
    height: 25px;
    border-radius: 13px;
    background: var(--surface3);
    border: 1px solid var(--border2);
    position: relative;
    flex: none;
    transition: background 0.2s ease;
  }
  .switch span {
    position: absolute;
    left: 2px;
    top: 2px;
    width: 19px;
    height: 19px;
    border-radius: 50%;
    background: var(--muted);
    transition: transform 0.2s ease, background 0.2s ease;
  }
  .switch.on {
    background: var(--accent);
    border-color: var(--accent);
  }
  .switch.on span {
    transform: translateX(19px);
    background: var(--accent-ink);
  }
  .about {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 16px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 14px;
  }
  .a-logo {
    font-size: 32px;
  }
  .about b {
    font-size: 15px;
  }
  .about p {
    font-size: 12px;
    color: var(--muted);
    margin-top: 2px;
  }
  .ver {
    font-size: 11px;
    color: var(--muted);
    font-weight: 600;
  }
  .note {
    font-size: 13px;
    color: var(--muted);
    margin-bottom: 14px;
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
</style>
