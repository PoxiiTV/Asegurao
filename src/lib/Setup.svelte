<script lang="ts">
  import { fly } from 'svelte/transition';
  import { api, type AuthKind } from './api';
  import { toast } from './toast.svelte';

  let { onDone }: { onDone: () => void } = $props();

  let step = $state(0);
  let kind = $state<AuthKind>('pin');
  let secret = $state('');
  let confirm = $state('');
  let code = $state('');
  let saved = $state(false);
  let level = $state('fortaleza');
  let busy = $state(false);

  const minLen = $derived(kind === 'pin' ? 6 : 8);
  const valid = $derived(
    secret.length >= minLen &&
      secret === confirm &&
      (kind === 'pin' ? /^\d+$/.test(secret) : true)
  );

  async function createPassword() {
    if (!valid) return;
    busy = true;
    try {
      code = await api.setup(kind, secret);
      step = 2;
    } catch (e) {
      toast(String(e), 'error');
    }
    busy = false;
  }

  async function finish() {
    busy = true;
    try {
      const snap = await api.snapshot();
      const s = snap.settings;
      s.level = level;
      await api.saveSettings(s);
      onDone();
    } catch (e) {
      toast(String(e), 'error');
    }
    busy = false;
  }

  function copyCode() {
    navigator.clipboard.writeText(code);
    toast('Código copiado', 'ok');
  }
</script>

<div class="screen">
  <div class="panel" in:fly={{ y: 12, duration: 300 }}>
    <div class="dots">
      {#each [0, 1, 2, 3] as i}
        <span class:on={i <= step}></span>
      {/each}
    </div>

    {#if step === 0}
      <div class="step" in:fly={{ x: 16, duration: 220 }}>
        <div class="logo">🛡️</div>
        <h1>Bienvenido a Asegurao</h1>
        <p class="sub">
          Protege tus aplicaciones con contraseña. Cuando alguien intente abrir una app
          protegida, Asegurao la congelará y pedirá la clave.
        </p>
        <button class="btn" onclick={() => (step = 1)}>Empezar</button>
      </div>
    {:else if step === 1}
      <div class="step" in:fly={{ x: 16, duration: 220 }}>
        <h1>Tu contraseña maestra</h1>
        <p class="sub">Es la llave para entrar a Asegurao y autorizar cambios.</p>

        <div class="seg">
          <button class:on={kind === 'pin'} onclick={() => (kind = 'pin')}>PIN</button>
          <button class:on={kind === 'password'} onclick={() => (kind = 'password')}
            >Contraseña</button
          >
        </div>

        <input
          class="field"
          type="password"
          inputmode={kind === 'pin' ? 'numeric' : 'text'}
          placeholder={kind === 'pin' ? 'Mínimo 6 dígitos' : 'Mínimo 8 caracteres'}
          bind:value={secret}
        />
        <input
          class="field"
          type="password"
          placeholder="Repite la contraseña"
          bind:value={confirm}
          onkeydown={(e) => e.key === 'Enter' && createPassword()}
        />
        {#if secret && confirm && secret !== confirm}
          <span class="warn">No coinciden</span>
        {/if}

        <div class="row">
          <button class="btn ghost" onclick={() => (step = 0)}>Atrás</button>
          <button class="btn" disabled={!valid || busy} onclick={createPassword}>Continuar</button>
        </div>
      </div>
    {:else if step === 2}
      <div class="step" in:fly={{ x: 16, duration: 220 }}>
        <h1>Código de recuperación</h1>
        <p class="sub">
          Si olvidas tu contraseña, este código es la <b>única</b> forma de recuperar el acceso.
          Guárdalo en un lugar seguro (papel mejor que digital).
        </p>
        <div class="code" onclick={copyCode} role="button" tabindex="0" onkeydown={() => {}}>
          {code}
          <span class="copy">copiar</span>
        </div>
        <label class="check">
          <input type="checkbox" bind:checked={saved} />
          He guardado el código en un lugar seguro
        </label>
        <div class="row">
          <button class="btn" disabled={!saved} onclick={() => (step = 3)}>Continuar</button>
        </div>
      </div>
    {:else}
      <div class="step" in:fly={{ x: 16, duration: 220 }}>
        <h1>Nivel de protección</h1>
        <p class="sub">Puedes cambiarlo cuando quieras desde Ajustes.</p>

        <button class="level" class:on={level === 'fortaleza'} onclick={() => (level = 'fortaleza')}>
          <div class="lv-ico">🏰</div>
          <div>
            <b>Fortaleza <span class="rec">recomendado</span></b>
            <p>Para desactivar la protección, desinstalar o cambiar la clave, Windows te pedirá tu
              contraseña además de la de Asegurao.</p>
          </div>
        </button>
        <button class="level" class:on={level === 'base'} onclick={() => (level = 'base')}>
          <div class="lv-ico">🔒</div>
          <div>
            <b>Base</b>
            <p>Protección estándar sin la confirmación extra de Windows.</p>
          </div>
        </button>

        <div class="row">
          <button class="btn" disabled={busy} onclick={finish}>Terminar</button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .screen {
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
  }
  .panel {
    width: 460px;
    max-width: 100%;
  }
  .dots {
    display: flex;
    gap: 7px;
    justify-content: center;
    margin-bottom: 26px;
  }
  .dots span {
    width: 26px;
    height: 5px;
    border-radius: 3px;
    background: var(--border2);
    transition: background 0.3s ease;
  }
  .dots span.on {
    background: var(--accent);
  }
  .step {
    text-align: center;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .logo {
    font-size: 52px;
  }
  h1 {
    font-size: 24px;
    font-weight: 800;
  }
  .sub {
    color: var(--muted);
    font-size: 14px;
    line-height: 1.5;
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
  .warn {
    color: var(--danger);
    font-size: 12px;
    text-align: left;
  }
  .row {
    display: flex;
    gap: 10px;
    margin-top: 8px;
  }
  .row .btn {
    flex: 1;
  }
  .code {
    font-family: 'Courier New', monospace;
    font-size: 22px;
    font-weight: 700;
    letter-spacing: 2px;
    padding: 18px;
    border-radius: 14px;
    background: var(--accent-soft);
    border: 1px dashed var(--accent);
    color: var(--accent);
    position: relative;
  }
  .copy {
    position: absolute;
    right: 10px;
    top: 8px;
    font-size: 10px;
    font-family: 'Plus Jakarta Sans';
    letter-spacing: 0;
    opacity: 0.6;
  }
  .check {
    display: flex;
    align-items: center;
    gap: 9px;
    font-size: 13px;
    color: var(--muted);
    justify-content: center;
  }
  .level {
    display: flex;
    gap: 13px;
    text-align: left;
    padding: 14px;
    border-radius: 14px;
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--text);
    transition: border-color 0.15s ease, background 0.15s ease;
  }
  .level.on {
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .lv-ico {
    font-size: 26px;
  }
  .level b {
    font-size: 14px;
  }
  .level p {
    font-size: 12px;
    color: var(--muted);
    margin-top: 3px;
    line-height: 1.4;
  }
  .rec {
    font-size: 10px;
    background: var(--accent);
    color: var(--accent-ink);
    padding: 2px 6px;
    border-radius: 6px;
    text-transform: uppercase;
    font-weight: 800;
    vertical-align: middle;
  }
</style>
