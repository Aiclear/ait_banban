<script lang="ts">
    import Fa from "svelte-fa";
    import {
        faClockRotateLeft,
        faXmark,
        faPlay,
        faPause,
        faCamera,
        faArrowLeft,
        faArrowRight,
    } from "@fortawesome/free-solid-svg-icons";
    import {
        snapshotState,
        enterTimeTravelMode,
        exitTimeTravelMode,
        goToSnapshot,
        createSnapshot,
        formatTime,
        getTimeUntilNextSave,
        loadSnapshots,
    } from "../../snapshot.svelte";

    let timeUntilNextSave = $state(0);
    let updateTimer: number | null = null;

    $effect(() => {
        updateTimeDisplay();
        loadSnapshots();

        updateTimer = window.setInterval(() => {
            updateTimeDisplay();
        }, 1000);

        return () => {
            if (updateTimer !== null) {
                clearInterval(updateTimer);
            }
        };
    });

    function updateTimeDisplay() {
        timeUntilNextSave = getTimeUntilNextSave();
    }

    function formatTimeRemaining(ms: number): string {
        const minutes = Math.floor(ms / 60000);
        const seconds = Math.floor((ms % 60000) / 1000);
        return `${minutes}:${seconds.toString().padStart(2, "0")}`;
    }

    function handleSliderInput(event: Event) {
        const target = event.target as HTMLInputElement;
        const index = parseInt(target.value);
        goToSnapshot(index);
    }

    function handlePrev() {
        if (snapshotState.currentSnapshotIndex === null) {
            if (snapshotState.snapshots.length > 0) {
                goToSnapshot(snapshotState.snapshots.length - 1);
            }
        } else if (snapshotState.currentSnapshotIndex < snapshotState.snapshots.length - 1) {
            goToSnapshot(snapshotState.currentSnapshotIndex + 1);
        }
    }

    function handleNext() {
        if (snapshotState.currentSnapshotIndex !== null && snapshotState.currentSnapshotIndex > 0) {
            goToSnapshot(snapshotState.currentSnapshotIndex - 1);
        }
    }

    async function handleManualSnapshot() {
        await createSnapshot();
    }
</script>

<div class="relative">
    {#if snapshotState.isTimeTravelMode}
        <div
            class="fixed top-0 left-0 right-0 z-50 bg-gradient-to-r from-purple-600 to-indigo-600 text-white shadow-lg"
        >
            <div class="flex items-center justify-between px-6 py-3">
                <div class="flex items-center gap-3">
                    <Fa icon={faClockRotateLeft} class="text-xl" />
                    <span class="font-semibold">时间回溯模式</span>
                    {#if snapshotState.snapshots.length > 0 && snapshotState.currentSnapshotIndex !== null}
                        <span class="text-sm opacity-80">
                            查看: {formatTime(snapshotState.snapshots[snapshotState.currentSnapshotIndex].createdAt)}
                        </span>
                    {/if}
                </div>
                <button
                    class="flex items-center gap-2 px-4 py-2 bg-white bg-opacity-20 rounded-lg hover:bg-opacity-30 transition-all"
                    onclick={exitTimeTravelMode}
                >
                    <Fa icon={faXmark} />
                    <span>退出</span>
                </button>
            </div>

            <div class="px-6 py-4 bg-black bg-opacity-20">
                <div class="flex items-center gap-4">
                    <button
                        class="w-10 h-10 flex items-center justify-center rounded-lg bg-white bg-opacity-20 hover:bg-opacity-30 disabled:opacity-50 disabled:cursor-not-allowed transition-all"
                        onclick={handlePrev}
                        disabled={snapshotState.currentSnapshotIndex === null ||
                            snapshotState.currentSnapshotIndex >= snapshotState.snapshots.length - 1}
                    >
                        <Fa icon={faArrowLeft} />
                    </button>

                    <div class="flex-1">
                        <input
                            type="range"
                            min="0"
                            max={Math.max(0, snapshotState.snapshots.length - 1)}
                            value={snapshotState.currentSnapshotIndex ?? 0}
                            oninput={handleSliderInput}
                            class="w-full h-2 bg-white bg-opacity-30 rounded-lg appearance-none cursor-pointer accent-white"
                        />
                        <div class="flex justify-between mt-2 text-xs opacity-70">
                            <span>
                                {snapshotState.snapshots.length > 0
                                    ? formatTime(snapshotState.snapshots[snapshotState.snapshots.length - 1].createdAt)
                                    : "无快照"}
                            </span>
                            <span>
                                {snapshotState.snapshots.length > 0
                                    ? formatTime(snapshotState.snapshots[0].createdAt)
                                    : ""}
                            </span>
                        </div>
                    </div>

                    <button
                        class="w-10 h-10 flex items-center justify-center rounded-lg bg-white bg-opacity-20 hover:bg-opacity-30 disabled:opacity-50 disabled:cursor-not-allowed transition-all"
                        onclick={handleNext}
                        disabled={snapshotState.currentSnapshotIndex === null ||
                            snapshotState.currentSnapshotIndex <= 0}
                    >
                        <Fa icon={faArrowRight} />
                    </button>
                </div>

                {#if snapshotState.snapshots.length > 0}
                    <div class="mt-3 flex gap-2 overflow-x-auto pb-1">
                        {#each snapshotState.snapshots as snapshot, index}
                            <button
                                class="flex-shrink-0 px-3 py-1.5 text-xs rounded-lg transition-all {snapshotState.currentSnapshotIndex === index
                                    ? 'bg-white text-purple-600 font-medium'
                                    : 'bg-white bg-opacity-20 hover:bg-opacity-30'}"
                                onclick={() => goToSnapshot(index)}
                            >
                                {formatTime(snapshot.createdAt)}
                            </button>
                        {/each}
                    </div>
                {/if}
            </div>
        </div>
    {:else}
        <div class="flex items-center gap-2">
            {#if snapshotState.autoSaveEnabled}
                <div class="flex items-center gap-2 px-3 py-1.5 bg-secondary-bg-token rounded-lg text-sm text-tertiary-txt-token">
                    <Fa icon={faClockRotateLeft} class="text-primary-500" />
                    <span>下次保存: {formatTimeRemaining(timeUntilNextSave)}</span>
                </div>
            {/if}

            <button
                class="flex items-center gap-2 px-3 py-1.5 bg-secondary-bg-token hover:bg-secondary-hover-token rounded-lg text-sm text-tertiary-txt-token transition-all"
                onclick={handleManualSnapshot}
                title="立即保存快照"
            >
                <Fa icon={faCamera} />
                <span>保存</span>
            </button>

            <button
                class="flex items-center gap-2 px-3 py-1.5 bg-primary-500 hover:bg-primary-600 text-white rounded-lg text-sm transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                onclick={enterTimeTravelMode}
                disabled={snapshotState.snapshots.length === 0}
                title="时间回溯 - 查看历史快照"
            >
                <Fa icon={faClockRotateLeft} />
                <span>时间回溯</span>
                {#if snapshotState.snapshots.length > 0}
                    <span class="bg-white bg-opacity-20 px-1.5 py-0.5 rounded text-xs">
                        {snapshotState.snapshots.length}
                    </span>
                {/if}
            </button>
        </div>
    {/if}
</div>

<style>
    input[type="range"] {
        -webkit-appearance: none;
        appearance: none;
        background: transparent;
        cursor: pointer;
    }

    input[type="range"]::-webkit-slider-runnable-track {
        background: rgba(255, 255, 255, 0.3);
        height: 8px;
        border-radius: 4px;
    }

    input[type="range"]::-moz-range-track {
        background: rgba(255, 255, 255, 0.3);
        height: 8px;
        border-radius: 4px;
    }

    input[type="range"]::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 20px;
        height: 20px;
        border-radius: 50%;
        background: white;
        margin-top: -6px;
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.3);
    }

    input[type="range"]::-moz-range-thumb {
        width: 20px;
        height: 20px;
        border-radius: 50%;
        background: white;
        border: none;
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.3);
    }
</style>
