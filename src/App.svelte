<script lang="ts">
    import { AppShell, initializeStores, Modal } from "@skeletonlabs/skeleton";
    import Board from "./lib/components/board/Board.svelte";
    import BoardDrawer from "./lib/components/drawer/BoardDrawer.svelte";
    import BoardSidebar from "./lib/components/board/BoardSidebar.svelte";
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";

    import { Toast } from "@skeletonlabs/skeleton";
    import { fetchAll, fetchAllBoards, setCurrentBoardId } from "./lib/shared.svelte";

    initializeStores();

    onMount(async () => {
        try {
            const boards = await fetchAllBoards();
            if (boards.length > 0) {
                setCurrentBoardId(boards[0].id);
            }
            await fetchAll();
        } catch (e) {
            console.error("Failed to initialize data:", e);
        } finally {
            try {
                await invoke("close_splashscreen");
            } catch (e) {
                console.error("Failed to close splashscreen:", e);
            }
        }
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
