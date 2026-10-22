<script lang="ts">
  import { faceOf, type Roster } from "./students";

  let { roster, name }: { roster: Roster; name: string } = $props();

  const src = $derived(faceOf(roster, name));
</script>

<!-- The name is written beside the face, so the picture says nothing more. -->
<span class="face">
  {#if src}<img {src} alt="" width="28" height="28" />{/if}
</span>

<style lang="sass">
  .face
    flex: none
    width: 28px
    height: 28px
    overflow: hidden
    border: 1px solid var(--c-border)
    border-radius: var(--radius-sm)
    background: var(--c-wash-base)

    img
      display: block
      width: 100%
      height: 100%
</style>
