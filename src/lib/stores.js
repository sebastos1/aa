import { writable, derived } from "svelte/store";
import { clientSettings } from "./clientSettings.js";

// should never change (probably)
export const advancements = writable({});
export const categories = writable({});
export const world = writable({});
export const players = writable({}); // advancement progress data
export const progress = writable({});

export const selectedPlayer = derived(
    [players, clientSettings],
    ([$players, $clientSettings]) => {
        if ($clientSettings.selectedPlayerUuid) {
            return $players[$clientSettings.selectedPlayerUuid] || null;
        }
        return null;
    }
);