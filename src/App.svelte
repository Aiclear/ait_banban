<script lang="ts">
    import { AppShell, initializeStores, Modal } from "@skeletonlabs/skeleton";
    import Board from "./lib/components/board/Board.svelte";
    import BoardDrawer from "./lib/components/drawer/BoardDrawer.svelte";
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";

    import { Toast } from "@skeletonlabs/skeleton";
    import { fetchAll } from "./lib/shared.svelte";
    import { initSnapshotSystem } from "./lib/snapshot.svelte";

    initializeStores();

    onMount(async () => {
        await fetchAll();
        initSnapshotSystem();
        await invoke("close_splashscreen");
    });
</script>

<div style="display: contents" class="h-full overflow-hidden">
    <BoardDrawer />
    <Modal />
    <Toast />
    <AppShell>
        <Board />
    </AppShell>
</div>
