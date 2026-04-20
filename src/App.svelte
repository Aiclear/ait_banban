<script lang="ts">
    import { AppShell, initializeStores, Modal } from "@skeletonlabs/skeleton";
    import Board from "./lib/components/board/Board.svelte";
    import BoardDrawer from "./lib/components/drawer/BoardDrawer.svelte";
    import BoardSidebar from "./lib/components/board/BoardSidebar.svelte";
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";

    import { Toast } from "@skeletonlabs/skeleton";
    import { fetchAll, fetchAllBoards, currentBoardId, boardsRune } from "./lib/shared.svelte";

    initializeStores();

    onMount(async () => {
        await fetchAllBoards();
        await fetchAll();
        await invoke("close_splashscreen");
    });
</script>

<div style="display: contents" class="h-full overflow-hidden">
    <BoardDrawer />
    <BoardSidebar />
    <Modal />
    <Toast />
    <AppShell>
        <Board />
    </AppShell>
</div>
