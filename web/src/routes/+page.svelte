<script>
	import Header from "$lib/components/Header.svelte";
	import Advancement from "$lib/components/Advancement.svelte";
	import { clientSettings } from "$lib/clientSettings.js";
	import { players } from "$lib/stores.js";

	const { data } = $props();
	const { advancements, categories, world } = data;

	const selectedPlayerUuid = $derived($clientSettings.selectedPlayerUuid);
	const advancementList = $derived(Object.values(advancements));
</script>

<Header {world} />

<main>
	<div>
		<div onclick={() => clientSettings.toggleCoopMode()} role="button" tabindex="0">
			CLICK HERE TO TOGGLE
		</div>
		<p>Current state: <strong>{$clientSettings.coopMode ? "ON" : "OFF"}</strong></p>
		{#if $clientSettings.coopMode}
			<p>THE FLAG IS ON, wave it</p>
		{/if}
	</div>

	<div class="advancement-grid">
		{#each advancementList as advancement (advancement.key)}
			{@const category = (data.categories || {})[advancement.category]}
			<Advancement {advancement} {selectedPlayerUuid} {category} showTitle={false} />
		{/each}
	</div>

</main>

<style>
	main {
		width: 100%;
		max-width: 75rem;
		margin: 0 auto;
	}
	
	.advancement-grid {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
		justify-content: center;
	}
</style>