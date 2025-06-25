import { writable } from "svelte/store";

export const advancements = writable({});
export const players = writable({}); // advancement progress data
export const categories = writable({});
export const world = writable({});