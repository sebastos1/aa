<script>
	import '../app.css';
	import { advancements, players, categories, world } from '$lib/stores.js';

	let { data, children } = $props();

	$advancements = data.advancements;
	$players = data.players;
	$categories = data.categories;
	$world = data.world;
</script>

{@render children()}
