<script>
	import IconText from "./IconText.svelte";
	import PlayerIcon from "./PlayerIcon.svelte";
	import { PLACEHOLDERS } from "$lib/utils.js";
	import PlayerDropdown from "./PlayerDropdown.svelte";

	import { players } from "$lib/stores.js";
	import { clientSettings } from "$lib/clientSettings.js";

	let { world } = $props();
	const worldIconUrl = $derived(world?.iconPath);

	const playerList = $derived(Object.values($players));
	const playerCount = $derived(playerList.length);
	const singlePlayer = $derived(playerCount <= 1 ? playerList[0] : null);
</script>

<header>
	<h1>
		<span>Advancements in</span>
		<IconText src={worldIconUrl ?? PLACEHOLDERS.worldIcon} text={world?.name ?? "World"} />
		<span>for</span>
        {#if singlePlayer}
			<PlayerIcon player={singlePlayer} />
		{:else}
			<PlayerDropdown />
		{/if}
	</h1>
</header>

<style>
	header {
		background-color: #111;
		padding: 1rem 0;
		border-bottom: 2px solid #555;
		margin-bottom: 1rem;
        justify-items: center;
	}
	h1 {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 0.75rem;
		font-size: 1.75rem;
	}
</style>