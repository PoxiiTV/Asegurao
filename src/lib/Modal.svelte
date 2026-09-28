<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import type { Snippet } from 'svelte';

  let {
    title = '',
    onclose,
    children,
    wide = false
  }: { title?: string; onclose?: () => void; children: Snippet; wide?: boolean } = $props();
</script>

<div
  class="backdrop"
  transition:fade={{ duration: 160 }}
  onclick={onclose}
  onkeydown={(e) => e.key === 'Escape' && onclose?.()}
  role="button"
  tabindex="-1"
>
  <div
    class="modal"
    class:wide
    transition:scale={{ duration: 200, start: 0.94 }}
    onclick={(e) => e.stopPropagation()}
    role="dialog"
    tabindex="-1"
  >
    {#if title}
      <div class="head">
        <h3>{title}</h3>
        <button class="x" onclick={onclose} aria-label="Cerrar">✕</button>
      </div>
    {/if}
    <div class="body">
      {@render children()}
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 500;
    padding: 20px;
  }
  .modal {
    width: 440px;
    max-width: 100%;
    max-height: 88vh;
    overflow: auto;
    background: var(--app-bg);
    background-attachment: fixed;
    border: 1px solid var(--border2);
    border-radius: 18px;
    box-shadow: var(--shadow);
  }
  .modal.wide {
    width: 620px;
  }
  .head {
    display: flex;
    align-items: center;
    padding: 18px 20px 0;
  }
  .head h3 {
    font-size: 17px;
    font-weight: 700;
  }
  .x {
    margin-left: auto;
    background: var(--surface2);
    border: 1px solid var(--border);
    color: var(--muted);
    width: 30px;
    height: 30px;
    border-radius: 9px;
    font-size: 13px;
  }
  .x:hover {
    background: var(--surface3);
    color: var(--text);
  }
  .body {
    padding: 18px 20px 20px;
  }
</style>
