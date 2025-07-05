<script>
	import { PLACEHOLDERS } from "$lib/utils.js";
	import IconText from "./IconText.svelte";

	let { player } = $props();

	const displayName = $derived(player?.name ?? player?.uuid?.split("-")[0] ?? "Unknown");
	const avatarSrc = $derived(player?.avatarUrl ?? PLACEHOLDERS.face);
</script>

<IconText src={avatarSrc} text={displayName} />