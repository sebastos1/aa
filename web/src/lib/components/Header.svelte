<script>
	import PlayerDropdown from './PlayerDropdown.svelte';
	import IconText from './IconText.svelte';
	import PlayerIcon from './PlayerIcon.svelte';
	import { DEFAULTS } from '$lib/utils.js';

	let { world, players, selectedPlayerUuid, onPlayerChange } = $props();
	const worldIconUrl = $derived(world?.icon_path);

    // players = players.slice(0, 1);

    const singlePlayer = $derived(players[0]);
</script>

<header>
	<h1>
		<span>Advancements for</span>
        {#if players.length === 1}
			<PlayerIcon player={singlePlayer} />
		{:else}
            <PlayerDropdown {players} {selectedPlayerUuid} onSelect={onPlayerChange} />
		{/if}
		<span>in</span>
		<IconText src={worldIconUrl ?? DEFAULTS.world_icon} text={world?.name ?? '...'} />
	</h1>
</header>

<style>
	header {
		background-color: #111;
		padding: 1rem 2rem;
		border-bottom: 2px solid #555;
		margin-bottom: 2rem;
        justify-items: center;
	}
	h1 {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 0.75rem;
		margin: 0;
		font-size: 1.5rem;
	}
</style>