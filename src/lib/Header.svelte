<script lang="ts">
  import Icon from "./Icon.svelte";
  import ThemeModal from "./ThemeModal.svelte";

  let {
    onnew,
    onopen,
    onsaveas,
  }: { onnew: () => void; onopen: () => void; onsaveas: () => void } = $props();

  let menuOpen = $state(false);
  let themeOpen = $state(false);
  let menuButton = $state<HTMLButtonElement | undefined>();

  function closeMenu() {
    menuOpen = false;
    menuButton?.focus();
  }

  function choose(action: () => void) {
    menuOpen = false;
    action();
  }

  function closeTheme() {
    themeOpen = false;
    menuButton?.focus();
  }

  function onkeydown(event: KeyboardEvent) {
    if (menuOpen && event.key === "Escape") {
      event.preventDefault();
      closeMenu();
    }
  }
</script>

<svelte:window {onkeydown} />

<header>
  <span class="title">Timeline Arona</span>
  <div class="menu-wrapper">
    <button
      class="icon-btn"
      type="button"
      aria-label="メニュー"
      aria-expanded={menuOpen}
      bind:this={menuButton}
      onclick={() => (menuOpen = !menuOpen)}
    >
      <Icon name="menu" />
    </button>
    {#if menuOpen}
      <button
        class="menu-overlay"
        type="button"
        tabindex="-1"
        aria-label="メニューを閉じる"
        onclick={closeMenu}
      ></button>
      <nav class="menu">
        <button class="menu-item" type="button" onclick={() => choose(() => (themeOpen = true))}>
          テーマ設定
        </button>
        <button class="menu-item" type="button" onclick={() => choose(onnew)}>
          新規 <kbd>Ctrl+N</kbd>
        </button>
        <button class="menu-item" type="button" onclick={() => choose(onopen)}>
          開く… <kbd>Ctrl+O</kbd>
        </button>
        <button class="menu-item" type="button" onclick={() => choose(onsaveas)}>
          名前を付けて保存… <kbd>Ctrl+Shift+S</kbd>
        </button>
      </nav>
    {/if}
  </div>
</header>

{#if themeOpen}
  <ThemeModal onclose={closeTheme} />
{/if}

<style lang="sass">
  header
    position: sticky
    top: 0
    z-index: 10
    display: flex
    align-items: center
    justify-content: space-between
    height: var(--header-h)
    padding: 0 var(--sp-3)
    background: var(--c-wash-base)
    border-bottom: 1px solid var(--c-border)

  .title
    font-size: var(--fs-md)
    font-weight: 500
    color: var(--c-on-surface)

  .menu-wrapper
    position: relative
    display: flex
    align-items: center
    align-self: stretch

  .menu-overlay
    position: fixed
    inset: 0
    z-index: 19
    padding: 0
    border: none
    background: transparent
    cursor: default

  .menu
    position: absolute
    top: 100%
    right: 0
    z-index: 20
    display: flex
    flex-direction: column
    min-width: 180px
    overflow: hidden
    border: 1px solid var(--c-border)
    border-radius: var(--radius-lg)
    background: var(--c-surface-raised)
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.25)

  .menu-item
    display: flex
    justify-content: space-between
    gap: var(--sp-4)
    width: 100%
    padding: var(--sp-2) var(--sp-3)
    border: none
    background: transparent
    color: var(--c-on-surface)
    font-size: var(--fs-md)
    font-weight: 500
    text-align: left
    white-space: nowrap
    cursor: pointer

    &:hover
      background: var(--c-hover-1)

  kbd
    color: var(--c-muted)
    font-family: inherit
    font-size: var(--fs-xs)
    font-weight: 400
</style>
