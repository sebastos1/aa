<script>
	import Advancement from '$lib/components/Advancement.svelte';
	import Header from '$lib/components/Header.svelte';

	let { data } = $props();
	const { advancements, players, categories, world } = data;

	let selectedPlayerUuid = $state(Object.keys(players)[0] ?? '');
	const selectedPlayer = $derived(players[selectedPlayerUuid]);
	const advancementList = $derived(Object.values(advancements));

	function handlePlayerChange(newUuid) {
		selectedPlayerUuid = newUuid;
	}
</script>

<Header {world} players={Object.values(players)} {selectedPlayerUuid} onPlayerChange={handlePlayerChange} />

<div class="advancement-grid">
	{#each advancementList as advancement (advancement.key)}
		{@const progress = selectedPlayer?.advancement_progress[advancement.key]}
		{@const category = (data.categories || {})[advancement.category]}
		<Advancement {advancement} {progress} {category} showTitle={false} />
	{/each}
</div>

<style>
	.advancement-grid {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
		justify-content: center;
	}
</style>