<script lang="ts">
  import { fly } from 'svelte/transition';
  import { toasts } from './toast.svelte';
</script>

<div class="wrap">
  {#each toasts as t (t.id)}
    <div class="toast {t.kind}" transition:fly={{ y: 20, duration: 220 }}>
      <span class="dot"></span>{t.msg}
    </div>
  {/each}
</div>

<style>
  .wrap {
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: center;
    z-index: 9999;
    pointer-events: none;
  }
  .toast {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 11px 16px;
    border-radius: 12px;
    background: var(--surface3);
    border: 1px solid var(--border2);
    backdrop-filter: blur(20px);
    box-shadow: var(--shadow);
    font-size: 13px;
    font-weight: 600;
    max-width: 460px;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--accent);
    flex: none;
  }
  .ok .dot {
    background: var(--ok);
  }
  .error .dot {
    background: var(--danger);
  }
</style>
