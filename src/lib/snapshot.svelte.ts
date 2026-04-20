import { invoke } from "@tauri-apps/api/core";
import type {
    SnapshotInfo,
    SnapshotData,
    BoardSnapshot,
    ColumnSnapshot,
    ActivitySnapshot,
    OtherActivitySnapshot,
    CategorySnapshot,
    CategoryTagSnapshot,
    OtherTagSnapshot,
} from "./interfaces";
import {
    categoriesRune,
    categoryTagsRune,
    otherTagsRune,
    activitiesRune,
    columnsRune,
    otherActivitiesRune,
    draggableColumns,
    draggableActivities,
    draggableOtherActivities,
    fetchAll,
} from "./shared.svelte";

const SNAPSHOT_INTERVAL_MS = 3 * 60 * 1000;
const MAX_SNAPSHOTS = 30;

export interface SnapshotState {
    snapshots: SnapshotInfo[];
    isTimeTravelMode: boolean;
    currentSnapshotIndex: number | null;
    autoSaveEnabled: boolean;
    lastSaveTime: Date | null;
    nextSaveTime: Date | null;
}

export const snapshotState: SnapshotState = $state({
    snapshots: [],
    isTimeTravelMode: false,
    currentSnapshotIndex: null,
    autoSaveEnabled: true,
    lastSaveTime: null,
    nextSaveTime: null,
});

let autoSaveTimer: number | null = null;

export async function loadSnapshots(): Promise<void> {
    try {
        const snapshots = (await invoke("get_all_snapshots")) as SnapshotInfo[];
        snapshotState.snapshots = snapshots;
    } catch (error) {
        console.error("Failed to load snapshots:", error);
    }
}

function createBoardSnapshot(): BoardSnapshot {
    const columns: Record<number, ColumnSnapshot> = {};
    Object.entries(columnsRune).forEach(([columnId, column]) => {
        columns[+columnId] = {
            name: column.name,
            ordinal: column.ord,
            activities: [...column.activities],
        };
    });

    const activities: Record<number, ActivitySnapshot> = {};
    Object.entries(activitiesRune).forEach(([activityId, activity]) => {
        let columnId: number | null = null;
        for (const [colId, col] of Object.entries(columnsRune)) {
            if (col.activities.includes(+activityId)) {
                columnId = +colId;
                break;
            }
        }
        if (columnId !== null) {
            activities[+activityId] = {
                name: activity.name,
                body: activity.body,
                ordinal: activity.ordinal,
                tags: [...activity.tags],
                columnId: columnId,
            };
        }
    });

    const otherActivities: Record<number, OtherActivitySnapshot> = {};
    Object.entries(otherActivitiesRune.inner).forEach(([activityId, activity]) => {
        otherActivities[+activityId] = {
            name: activity.name,
            body: activity.body,
            ordinal: activity.ordinal,
            tags: [...activity.tags],
        };
    });

    const categories: Record<number, CategorySnapshot> = {};
    Object.entries(categoriesRune).forEach(([categoryId, category]) => {
        categories[+categoryId] = {
            name: category.name,
            ordinal: category.ord,
            tags: [...category.tags],
        };
    });

    const categoryIds: Record<number, number> = {};
    Object.entries(categoriesRune).forEach(([categoryId, category]) => {
        category.tags.forEach((tagId) => {
            categoryIds[tagId] = +categoryId;
        });
    });

    const categoryTags: Record<number, CategoryTagSnapshot> = {};
    Object.entries(categoryTagsRune).forEach(([tagId, tag]) => {
        categoryTags[+tagId] = {
            name: tag.name,
            color: tag.color,
            ordinal: tag.ord,
        };
    });

    const otherTags: Record<number, OtherTagSnapshot> = {};
    Object.entries(otherTagsRune).forEach(([tagId, tag]) => {
        otherTags[+tagId] = {
            name: tag.name,
            color: tag.color,
            ordinal: tag.ord,
        };
    });

    return {
        columns,
        activities,
        otherActivities,
        categories,
        categoryTags,
        otherTags,
    };
}

function restoreBoardSnapshot(snapshot: BoardSnapshot): void {
    Object.keys(categoriesRune).forEach((key) => delete categoriesRune[+key]);
    Object.keys(categoryTagsRune).forEach((key) => delete categoryTagsRune[+key]);
    Object.keys(otherTagsRune).forEach((key) => delete otherTagsRune[+key]);
    Object.keys(activitiesRune).forEach((key) => delete activitiesRune[+key]);
    Object.keys(columnsRune).forEach((key) => delete columnsRune[+key]);
    Object.keys(otherActivitiesRune.inner).forEach((key) => delete otherActivitiesRune.inner[+key]);

    const categoryIds: Record<number, number> = {};
    Object.entries(snapshot.categories).forEach(([categoryId, category]) => {
        categoriesRune[+categoryId] = {
            name: category.name,
            ord: category.ordinal,
            tags: [...category.tags],
        };
        category.tags.forEach((tagId) => {
            categoryIds[tagId] = +categoryId;
        });
    });

    Object.entries(snapshot.categoryTags).forEach(([tagId, tag]) => {
        categoryTagsRune[+tagId] = {
            name: tag.name,
            color: tag.color,
            ord: tag.ordinal,
            categoryId: categoryIds[+tagId] || 0,
        };
    });

    Object.entries(snapshot.otherTags).forEach(([tagId, tag]) => {
        otherTagsRune[+tagId] = {
            name: tag.name,
            color: tag.color,
            ord: tag.ordinal,
        };
    });

    Object.entries(snapshot.columns).forEach(([columnId, column]) => {
        columnsRune[+columnId] = {
            name: column.name,
            ord: column.ordinal,
            activities: [...column.activities],
        };
    });

    Object.entries(snapshot.activities).forEach(([activityId, activity]) => {
        activitiesRune[+activityId] = {
            name: activity.name,
            body: activity.body,
            ordinal: activity.ordinal,
            tags: [...activity.tags],
        };
    });

    Object.entries(snapshot.otherActivities).forEach(([activityId, activity]) => {
        otherActivitiesRune.inner[+activityId] = {
            name: activity.name,
            body: activity.body,
            ordinal: activity.ordinal,
            tags: [...activity.tags],
        };
    });

    draggableColumns.update();
    Object.keys(columnsRune).forEach((colId) => {
        draggableActivities.update(+colId);
    });
    draggableOtherActivities.update();
}

export async function createSnapshot(): Promise<SnapshotInfo | null> {
    if (snapshotState.isTimeTravelMode) {
        return null;
    }

    try {
        const boardSnapshot = createBoardSnapshot();
        const snapshotData = JSON.stringify(boardSnapshot);
        const createdAt = new Date().toISOString();

        const result = (await invoke("create_snapshot", {
            data: {
                snapshotData,
                createdAt,
            },
        })) as SnapshotInfo;

        snapshotState.lastSaveTime = new Date(createdAt);
        updateNextSaveTime();

        await loadSnapshots();

        console.log("Snapshot created:", result);
        return result;
    } catch (error) {
        console.error("Failed to create snapshot:", error);
        return null;
    }
}

export async function getSnapshot(id: number): Promise<SnapshotData | null> {
    try {
        const snapshot = (await invoke("get_snapshot", { id })) as SnapshotData | null;
        return snapshot;
    } catch (error) {
        console.error("Failed to get snapshot:", error);
        return null;
    }
}

export async function deleteSnapshot(id: number): Promise<void> {
    try {
        await invoke("delete_snapshot", { id });
        await loadSnapshots();
    } catch (error) {
        console.error("Failed to delete snapshot:", error);
    }
}

export async function loadSnapshotForPreview(index: number): Promise<void> {
    if (index < 0 || index >= snapshotState.snapshots.length) {
        return;
    }

    const snapshotInfo = snapshotState.snapshots[index];
    const snapshotData = await getSnapshot(snapshotInfo.id);

    if (!snapshotData) {
        return;
    }

    try {
        const boardSnapshot = JSON.parse(snapshotData.snapshotData) as BoardSnapshot;
        restoreBoardSnapshot(boardSnapshot);
        snapshotState.currentSnapshotIndex = index;
    } catch (error) {
        console.error("Failed to parse snapshot data:", error);
    }
}

export async function enterTimeTravelMode(): Promise<void> {
    if (snapshotState.isTimeTravelMode) {
        return;
    }

    snapshotState.isTimeTravelMode = true;
    stopAutoSave();

    if (snapshotState.snapshots.length > 0) {
        await loadSnapshotForPreview(0);
    }
}

export async function exitTimeTravelMode(): Promise<void> {
    if (!snapshotState.isTimeTravelMode) {
        return;
    }

    snapshotState.isTimeTravelMode = false;
    snapshotState.currentSnapshotIndex = null;

    await fetchAll();

    if (snapshotState.autoSaveEnabled) {
        startAutoSave();
    }
}

export async function goToSnapshot(index: number): Promise<void> {
    if (!snapshotState.isTimeTravelMode) {
        await enterTimeTravelMode();
    }
    await loadSnapshotForPreview(index);
}

export function startAutoSave(): void {
    if (autoSaveTimer !== null) {
        clearInterval(autoSaveTimer);
    }

    snapshotState.autoSaveEnabled = true;
    updateNextSaveTime();

    autoSaveTimer = window.setInterval(() => {
        if (!snapshotState.isTimeTravelMode) {
            createSnapshot();
        }
    }, SNAPSHOT_INTERVAL_MS);
}

export function stopAutoSave(): void {
    if (autoSaveTimer !== null) {
        clearInterval(autoSaveTimer);
        autoSaveTimer = null;
    }
    snapshotState.autoSaveEnabled = false;
    snapshotState.nextSaveTime = null;
}

function updateNextSaveTime(): void {
    if (snapshotState.autoSaveEnabled) {
        const now = new Date();
        snapshotState.nextSaveTime = new Date(now.getTime() + SNAPSHOT_INTERVAL_MS);
    }
}

export function formatTime(isoString: string): string {
    const date = new Date(isoString);
    return date.toLocaleString("zh-CN", {
        year: "numeric",
        month: "2-digit",
        day: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
    });
}

export function getTimeUntilNextSave(): number {
    if (!snapshotState.nextSaveTime) {
        return 0;
    }
    const now = new Date();
    const diff = snapshotState.nextSaveTime.getTime() - now.getTime();
    return Math.max(0, diff);
}

export function initSnapshotSystem(): void {
    loadSnapshots();
    startAutoSave();
}
