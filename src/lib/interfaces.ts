export interface Activity {
    name: string;
    body?: string;
    tags: Array<number>;
    ordinal: number;
}

export interface Column {
    name: string;
    activities: number[];
    ord: number;
}

export interface Tag {
    name: string;
    color: string;
    ord: number;
}

export interface Category {
    name: string;
    tags: Array<number>;
    ord: number;
}

export interface Editable {
    id: number;
    field: ActiveField;
    oldName: string;
}

export enum ActiveField {
    ActivityName = "activity name",
    ActivityBody = "activity body",
    ColumnName = "column name",
}

export enum DrawerTab {
    Activity = "activity",
    Settings = "settings",
    OtherActivities = "otherActivities",
}

export interface AppState {
    isDebug: boolean;
    previousDrawerTab: DrawerTab | null;
    currentEditable: Editable | null;
    selectedActivity: number | null;
    hoverColumnId: null | number;
}

export interface SnapshotInfo {
    id: number;
    createdAt: string;
}

export interface SnapshotData {
    id: number;
    createdAt: string;
    snapshotData: string;
}

export interface BoardSnapshot {
    columns: Record<number, ColumnSnapshot>;
    activities: Record<number, ActivitySnapshot>;
    otherActivities: Record<number, OtherActivitySnapshot>;
    categories: Record<number, CategorySnapshot>;
    categoryTags: Record<number, CategoryTagSnapshot>;
    otherTags: Record<number, OtherTagSnapshot>;
}

export interface ColumnSnapshot {
    name: string;
    ordinal: number;
    activities: number[];
}

export interface ActivitySnapshot {
    name: string;
    body?: string;
    ordinal: number;
    tags: number[];
    columnId: number;
}

export interface OtherActivitySnapshot {
    name: string;
    body?: string;
    ordinal: number;
    tags: number[];
}

export interface CategorySnapshot {
    name: string;
    ordinal: number;
    tags: number[];
}

export interface CategoryTagSnapshot {
    name: string;
    color: string;
    ordinal: number;
}

export interface OtherTagSnapshot {
    name: string;
    color: string;
    ordinal: number;
}
