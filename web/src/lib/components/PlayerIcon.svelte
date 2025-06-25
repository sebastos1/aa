<script>
	import { DEFAULTS } from '$lib/utils.js';
	import IconText from './IconText.svelte';

	let { player } = $props();

	const displayName = $derived(player?.name ?? player?.uuid?.split("-")[0] ?? "Unknown");
	const avatarSrc = $derived(player?.avatar_url ?? DEFAULTS.face);
</script>

<IconText src={avatarSrc} text={displayName} />