import { invoke } from "@tauri-apps/api/core";
import type { Activity, AppState, Board, Category, Column, Tag } from "./interfaces";
import { getToastStore, type ToastSettings, type ToastStore } from "@skeletonlabs/skeleton";

function clearObject<T extends Record<number, any>>(obj: T): void {
    for (const key of Object.keys(obj)) {
        delete obj[+key];
    }
}

export const appState: AppState = $state({
    isDebug: false,
    previousDrawerTab: null,
    currentEditable: null,
    selectedActivity: null,
    hoverColumnId: null,
});

const _boardsRune: Record<number, Board> = $state({});
let _currentBoardId: number | null = $state(null);

const _categoriesRune: Record<number, Category> = $state({});
const _categoryTagsRune: Record<number, Tag & { categoryId: number }> = $state({});
const _otherTagsRune: Record<number, Tag> = $state({});
const _activitiesRune: Record<number, Activity> = $state({});
const _columnsRune: Record<number, Column> = $state({});
export const otherActivitiesRune: { inner: Record<number, Activity> } = $state({ inner: {} });

export function getBoardsRune(): Record<number, Board> {
    return _boardsRune;
}

export function getCurrentBoardId(): number | null {
    return _currentBoardId;
}

export function getCategoriesRune(): Record<number, Category> {
    return _categoriesRune;
}

export function getCategoryTagsRune(): Record<number, Tag & { categoryId: number }> {
    return _categoryTagsRune;
}

export function getOtherTagsRune(): Record<number, Tag> {
    return _otherTagsRune;
}

export function getActivitiesRune(): Record<number, Activity> {
    return _activitiesRune;
}

export function getColumnsRune(): Record<number, Column> {
    return _columnsRune;
}

class IdTags {
    inner: { id: number; tag: Tag & { categoryId: number } }[][] = $state([]);

    update = () => {
        this.inner = Object.entries(_categoriesRune).map(([categoryId, category]) => {
            return category.tags
                .map((tagId) => {
                    const tag = _categoryTagsRune[tagId];
                    console.assert(tag !== undefined, "Category tag not found");
                    return { id: tagId, tag };
                })
                .sort((idTagA, idTagB) => {
                    return idTagA.tag.ord - idTagB.tag.ord;
                });
        });
    };
}

class IdOtherTags {
    inner: { id: number; tag: Tag }[] = $state([]);

    update = () => {
        this.inner = Object.entries(_otherTagsRune)
            .map(([id, tag]) => {
                return { id: +id, tag };
            })
            .sort((idTagA, idTagB) => {
                return idTagA.tag.ord - idTagB.tag.ord;
            });
    };
}

export class DraggableColumns {
    inner: { id: number; columnId: number; column: Column }[] = $state([]);

    update = () => {
        this.inner = Object.entries(_columnsRune)
            .map(([id, column]) => {
                return { id: +id, columnId: +id, column };
            })
            .sort((colA, colB) => {
                return colA.column.ord - colB.column.ord;
            });
    };
}

export class DraggableActivities {
    inner: Record<number, { id: number; colId: number; activity: Activity }[]> = $state({});

    update = (columnId: number) => {
        this.inner[columnId] = _columnsRune[columnId].activities
            .map((activityId) => {
                return { id: +activityId, colId: columnId, activity: _activitiesRune[+activityId] };
            })
            .sort((activity1, activity2) => {
                return activity1.activity.ordinal - activity2.activity.ordinal;
            });
    };
}

export class DraggableOtherActivities {
    inner: { id: number; activity: Activity }[] = $state([]);

    update = () => {
        this.inner = Object.entries(otherActivitiesRune.inner).map(([activityId, activity]) => {
            return { id: +activityId, activity };
        });
    };
}

export const idTags = new IdTags();
export const idOtherTags = new IdOtherTags();
export const draggableColumns = new DraggableColumns();
export const draggableActivities = new DraggableActivities();
export const draggableOtherActivities = new DraggableOtherActivities();

export async function changeCategoryTagColor(newColor: string, tagId: number) {
    const tag = _categoryTagsRune[tagId];
    await invoke("update_tag_color", {
        data: { categoryTagId: tagId, color: newColor.slice(1) },
    });
    tag.color = newColor;
    _categoryTagsRune[tagId] = tag;
}

export async function changeOtherTagColor(newColor: string, tagId: number) {
    const tag = _otherTagsRune[tagId];
    await invoke("update_tag_color", {
        data: { categoryTagId: tagId, color: newColor.slice(1) },
    });
    tag.color = newColor;
    _otherTagsRune[tagId] = tag;
}

export function showToast(toastStore: ToastStore, content: string) {
    const toast: ToastSettings = {
        message: content,
        hoverable: true,
        autohide: true,
        hideDismiss: true,
        timeout: 2000,
        classes: "variant-ghost-warning",
    };
    toastStore.trigger(toast);
}

interface BackendCol {
    name: string;
    ordinal: number;
    activities: Array<number>;
}

interface BackendOtherActv {
    name: string;
    body?: string;
    ordinal: number;
    tags: Array<number>;
}

interface BackendActv {
    name: string;
    body?: string;
    ordinal: number;
    tags: Array<number>;
    columnId: number;
}

interface BackendTag {
    name: string;
    color: string;
    ordinal: number;
}

interface BackendCategory {
    name: string;
    ordinal: number;
    tags: Array<number>;
}

interface RawLoadData {
    columns: Record<number, BackendCol>;
    activities: Record<number, BackendActv>;
    otherActivities: Record<number, BackendOtherActv>;
    categories: Record<number, BackendCategory>;
    categoryTags: Record<number, BackendTag>;
    otherTags: Record<number, BackendTag>;
}

export async function fetchAll() {
    const res = (await invoke("fetch_all")) as RawLoadData;
    const categoryIds: Record<number, number> = {};

    Object.entries(res.categories).forEach(([categoryId, category]) => {
        console.log(categoryId);
        _categoriesRune[+categoryId] = { ...category, ord: category.ordinal };
        category.tags.forEach((tagId) => {
            categoryIds[tagId] = +categoryId;
        });
    });

    Object.entries(res.categoryTags).forEach(([tagId, tag]) => {
        _categoryTagsRune[+tagId] = { ...tag, ord: tag.ordinal, categoryId: categoryIds[+tagId] };
    });

    Object.entries(res.otherTags).forEach(([tagId, tag]) => {
        _otherTagsRune[+tagId] = { ...tag, ord: tag.ordinal };
    });

    Object.entries(res.columns).forEach(([columnId, column]) => {
        _columnsRune[+columnId] = { ...column, ord: column.ordinal };
    });
    draggableColumns.update();

    Object.entries(res.activities).forEach(([activityId, activity]) => {
        _activitiesRune[+activityId] = activity;
    });

    Object.entries(res.otherActivities).forEach(([activityId, activity]) => {
        otherActivitiesRune.inner[+activityId] = activity;
    });
}

export async function fetchAllBoards(): Promise<Board[]> {
    const boards = (await invoke("get_all_boards")) as Board[];
    clearObject(_boardsRune);
    boards.forEach((board) => {
        _boardsRune[board.id] = board;
    });
    return boards;
}

export async function createBoard(name: string): Promise<Board> {
    const board = (await invoke("create_board", { data: { name } })) as Board;
    _boardsRune[board.id] = board;
    return board;
}

export async function updateBoard(id: number, name: string): Promise<Board> {
    const board = (await invoke("update_board", { data: { id, name } })) as Board;
    _boardsRune[board.id] = board;
    return board;
}

export async function deleteBoard(id: number): Promise<void> {
    await invoke("delete_board", { id });
    delete _boardsRune[id];
}

export async function duplicateBoard(sourceId: number, newName: string): Promise<Board> {
    const board = (await invoke("duplicate_board", { sourceId, newName })) as Board;
    _boardsRune[board.id] = board;
    return board;
}

export async function exportBoard(boardId: number) {
    const data = await invoke("export_board", { boardId });
    return JSON.stringify(data, null, 2);
}

export async function importBoard(
    name: string,
    jsonString: string
): Promise<Board> {
    const data = JSON.parse(jsonString);
    data.name = name;
    const board = (await invoke("import_board", { data })) as Board;
    _boardsRune[board.id] = board;
    return board;
}

export function clearCurrentBoardData() {
    clearObject(_categoriesRune);
    clearObject(_categoryTagsRune);
    clearObject(_otherTagsRune);
    clearObject(_activitiesRune);
    clearObject(_columnsRune);
    otherActivitiesRune.inner = {};
    draggableColumns.inner = [];
    draggableActivities.inner = {};
    draggableOtherActivities.inner = [];
    idTags.inner = [];
    idOtherTags.inner = [];
}

export function setCurrentBoardId(id: number | null) {
    _currentBoardId = id;
}

export async function switchToBoard(boardId: number) {
    clearCurrentBoardData();
    _currentBoardId = boardId;
    await fetchAll();
}

export function getCurrentBoardName(): string {
    if (_currentBoardId && _boardsRune[_currentBoardId]) {
        return _boardsRune[_currentBoardId].name;
    }
    return "Kanban";
}
