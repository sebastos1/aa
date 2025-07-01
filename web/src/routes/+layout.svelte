<script>
	import "../app.css";
	import { advancements, players, categories, world, progress } from "$lib/stores.js";
	import { clientSettings } from '$lib/clientSettings.js';
	import { browser } from "$app/environment";

	let { data, children } = $props();

	$advancements = data.advancements;
	$players = data.players;
	$categories = data.categories;
	$world = data.world;
	$progress = data.progress;

	$effect(() => {
		if (browser) {
			const availablePlayers = Object.keys($players);
			const currentUuid = $clientSettings.selectedPlayerUuid;

			if (availablePlayers.length > 0 && (!currentUuid || !availablePlayers.includes(currentUuid))) {
				clientSettings.setSelectedPlayer(availablePlayers[0]);
			} else if (availablePlayers.length === 0 && currentUuid) {
				clientSettings.setSelectedPlayer(null);
			}
		}
	});
</script>

{@render children()}