<script lang="ts">
    import { Modal, getModalStore, getToastStore } from "@skeletonlabs/skeleton";
    import { invoke } from "@tauri-apps/api/core";
    import { open, save } from "@tauri-apps/plugin-dialog";
    import { writeTextFile, readTextFile } from "@tauri-apps/plugin-fs";
    import type { Board } from "../../interfaces";
    import {
        getBoardsRune,
        getCurrentBoardId,
        fetchAllBoards,
        createBoard,
        updateBoard,
        deleteBoard,
        duplicateBoard,
        exportBoard,
        importBoard,
        switchToBoard,
        showToast,
        setCurrentBoardId,
    } from "../../shared.svelte";
    import Fa from "svelte-fa";
    import {
        faPlus,
        faEdit,
        faTrash,
        faCopy,
        faDownload,
        faUpload,
        faCheck,
        faTimes,
        faBars,
        faXmark,
        faKanban,
    } from "@fortawesome/free-solid-svg-icons";

    const modalStore = getModalStore();
    const toastStore = getToastStore();

    let isSidebarOpen = $state(false);
    let isCreating = $state(false);
    let newBoardName = $state("");
    let editingBoardId: number | null = $state(null);
    let editingBoardName = $state("");
    let isLoading = $state(false);

    async function loadBoards() {
        isLoading = true;
        try {
            const boards = await fetchAllBoards();
            if (boards.length > 0 && !getCurrentBoardId()) {
                setCurrentBoardId(boards[0].id);
            }
        } catch (e) {
            console.error("Failed to load boards:", e);
            showToast(toastStore, "加载看板列表失败");
        } finally {
            isLoading = false;
        }
    }

    async function handleCreateBoard() {
        if (!newBoardName.trim()) {
            showToast(toastStore, "请输入看板名称");
            return;
        }
        try {
            const board = await createBoard(newBoardName.trim());
            setCurrentBoardId(board.id);
            newBoardName = "";
            isCreating = false;
            showToast(toastStore, "看板创建成功");
        } catch (e) {
            console.error("Failed to create board:", e);
            showToast(toastStore, "创建看板失败");
        }
    }

    async function handleUpdateBoard() {
        if (!editingBoardId || !editingBoardName.trim()) {
            showToast(toastStore, "请输入看板名称");
            return;
        }
        try {
            await updateBoard(editingBoardId, editingBoardName.trim());
            editingBoardId = null;
            editingBoardName = "";
            showToast(toastStore, "看板更新成功");
        } catch (e) {
            console.error("Failed to update board:", e);
            showToast(toastStore, "更新看板失败");
        }
    }

    function startEdit(board: Board) {
        editingBoardId = board.id;
        editingBoardName = board.name;
    }

    function cancelEdit() {
        editingBoardId = null;
        editingBoardName = "";
    }

    function confirmDelete(board: Board) {
        modalStore.trigger({
            type: "confirm",
            title: "删除看板",
            body: `确定要删除看板"${board.name}"吗？此操作不可恢复。`,
            response: async (r: boolean) => {
                if (r) {
                    try {
                        await deleteBoard(board.id);
                        if (getCurrentBoardId() === board.id) {
                            const boards = Object.values(getBoardsRune());
                            setCurrentBoardId(boards.length > 0 ? boards[0].id : null);
                        }
                        showToast(toastStore, "看板删除成功");
                    } catch (e) {
                        console.error("Failed to delete board:", e);
                        showToast(toastStore, "删除看板失败");
                    }
                }
            },
        });
    }

    async function handleDuplicateBoard(board: Board) {
        try {
            const newBoard = await duplicateBoard(board.id, `${board.name} (复制)`);
            showToast(toastStore, `看板"${board.name}"复制成功`);
        } catch (e) {
            console.error("Failed to duplicate board:", e);
            showToast(toastStore, "复制看板失败");
        }
    }

    async function handleExportBoard(board: Board) {
        try {
            const jsonData = await exportBoard(board.id);
            const filePath = await save({
                title: "导出看板",
                defaultPath: `${board.name}.json`,
                filters: [
                    {
                        name: "JSON Files",
                        extensions: ["json"],
                    },
                ],
            });
            if (filePath) {
                await writeTextFile(filePath, jsonData);
                showToast(toastStore, `看板"${board.name}"导出成功`);
            }
        } catch (e) {
            console.error("Failed to export board:", e);
            showToast(toastStore, "导出看板失败");
        }
    }

    async function handleImportBoard() {
        try {
            const filePath = await open({
                title: "导入看板",
                multiple: false,
                filters: [
                    {
                        name: "JSON Files",
                        extensions: ["json"],
                    },
                ],
            });
            if (filePath) {
                const jsonData = await readTextFile(filePath as string);
                const parsed = JSON.parse(jsonData);
                const boardName = parsed.name || "导入的看板";
                const newBoard = await importBoard(boardName, jsonData);
                showToast(toastStore, `看板"${newBoard.name}"导入成功`);
            }
        } catch (e) {
            console.error("Failed to import board:", e);
            showToast(toastStore, "导入看板失败");
        }
    }

    async function handleSwitchBoard(boardId: number) {
        try {
            await switchToBoard(boardId);
            showToast(toastStore, `已切换到"${getBoardsRune()[boardId]?.name}"`);
        } catch (e) {
            console.error("Failed to switch board:", e);
            showToast(toastStore, "切换看板失败");
        }
    }

    function toggleSidebar() {
        isSidebarOpen = !isSidebarOpen;
    }

    $effect(() => {
        loadBoards();
    });
</script>

<button
    onclick={toggleSidebar}
    class="fixed top-4 left-4 z-50 btn variant-ghost-tertiary h-12 w-12 p-0 rounded-full shadow-lg bg-white bg-opacity-90"
    title="看板列表"
>
    <Fa icon={isSidebarOpen ? faXmark : faBars} />
</button>

<div
    class="fixed left-0 top-0 h-full w-72 bg-white shadow-xl z-40 transition-transform duration-300 flex flex-col {isSidebarOpen
        ? 'translate-x-0'
        : '-translate-x-full'}"
>
    <div class="p-4 border-b border-gray-200 flex items-center justify-between">
        <h2 class="text-lg font-bold text-gray-800 flex items-center gap-2">
            <Fa icon={faKanban} />
            看板列表
        </h2>
        <button
            onclick={handleImportBoard}
            class="btn variant-ghost-tertiary h-8 w-8 p-0"
            title="导入看板"
        >
            <Fa icon={faUpload} />
        </button>
    </div>

    <div class="flex-1 overflow-y-auto">
        {#if isLoading}
            <div class="p-4 text-center text-gray-500">加载中...</div>
        {:else if Object.keys(getBoardsRune()).length === 0}
            <div class="p-4 text-center text-gray-500">暂无看板</div>
        {:else}
            {#each Object.values(getBoardsRune()).sort((a, b) => a.id - b.id) as board}
                {#if editingBoardId === board.id}
                    <div class="p-3 border-b border-gray-100 bg-gray-50">
                        <div class="flex items-center gap-2 mb-2">
                            <input
                                bind:value={editingBoardName}
                                class="input input-sm flex-1"
                                placeholder="看板名称"
                            />
                        </div>
                        <div class="flex gap-2">
                            <button
                                onclick={handleUpdateBoard}
                                class="btn variant-ghost-success h-8 px-3 text-sm"
                            >
                                <Fa icon={faCheck} />
                            </button>
                            <button
                                onclick={cancelEdit}
                                class="btn variant-ghost-tertiary h-8 px-3 text-sm"
                            >
                                <Fa icon={faTimes} />
                            </button>
                        </div>
                    </div>
                {:else}
                    <div
                        class="p-3 border-b border-gray-100 hover:bg-gray-50 transition-colors {getCurrentBoardId() === board.id
                            ? 'bg-blue-50 border-l-4 border-l-blue-500'
                            : ''}"
                    >
                        <div
                            class="flex items-center justify-between cursor-pointer"
                            onclick={() => handleSwitchBoard(board.id)}
                        >
                            <span
                                class="font-medium text-gray-800 truncate flex-1 mr-2"
                                class:text-blue-600={getCurrentBoardId() === board.id}
                            >
                                {board.name}
                            </span>
                        </div>
                        <div class="flex gap-1 mt-2">
                            <button
                                onclick={(e) => {
                                    e.stopPropagation();
                                    startEdit(board);
                                }}
                                class="btn variant-ghost-tertiary h-7 px-2 text-xs"
                                title="重命名"
                            >
                                <Fa icon={faEdit} />
                            </button>
                            <button
                                onclick={(e) => {
                                    e.stopPropagation();
                                    handleDuplicateBoard(board);
                                }}
                                class="btn variant-ghost-tertiary h-7 px-2 text-xs"
                                title="复制"
                            >
                                <Fa icon={faCopy} />
                            </button>
                            <button
                                onclick={(e) => {
                                    e.stopPropagation();
                                    handleExportBoard(board);
                                }}
                                class="btn variant-ghost-tertiary h-7 px-2 text-xs"
                                title="导出"
                            >
                                <Fa icon={faDownload} />
                            </button>
                            <button
                                onclick={(e) => {
                                    e.stopPropagation();
                                    confirmDelete(board);
                                }}
                                class="btn variant-ghost-error h-7 px-2 text-xs"
                                title="删除"
                            >
                                <Fa icon={faTrash} />
                            </button>
                        </div>
                    </div>
                {/if}
            {/each}
        {/if}
    </div>

    <div class="p-4 border-t border-gray-200">
        {#if isCreating}
            <div class="flex flex-col gap-2">
                <input
                    bind:value={newBoardName}
                    class="input input-sm"
                    placeholder="输入看板名称"
                    onkeydown={(e) => e.key === "Enter" && handleCreateBoard()}
                />
                <div class="flex gap-2">
                    <button onclick={handleCreateBoard} class="btn variant-ghost-success h-9 flex-1">
                        <Fa icon={faCheck} />
                    </button>
                    <button
                        onclick={() => {
                            isCreating = false;
                            newBoardName = "";
                        }}
                        class="btn variant-ghost-tertiary h-9 flex-1"
                    >
                        <Fa icon={faTimes} />
                    </button>
                </div>
            </div>
        {:else}
            <button
                onclick={() => (isCreating = true)}
                class="btn variant-ghost-tertiary w-full h-10 flex items-center justify-center gap-2"
            >
                <Fa icon={faPlus} />
                新建看板
            </button>
        {/if}
    </div>
</div>

{#if isSidebarOpen}
    <div class="fixed inset-0 bg-black bg-opacity-20 z-30" onclick={toggleSidebar} />
{/if}

<Modal />
