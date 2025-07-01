export const CACHE_DIR = "caches"; // todo: env or something

export const PLACEHOLDERS = {
    face: "/defaults/steve.png",
    worldIcon: "/defaults/world-icon.png"
};

// get advancement icon
export function getIconUrl(icon) {
    if (!icon) return "/icons/minecraft_barrier.png";

    switch (icon.type) {
        case "playerHead":
            return icon.textureId ? `/heads/${icon.textureId}.png` : "/icons/minecraft_player_head.png";

        case "item":
            return icon.name ? `/icons/minecraft_${icon.name}.png` : "/icons/minecraft_stone.png";

        default:
            return "/icons/minecraft_barrier.png";
    }
}

export function getAdvancementTextColor(type) {
    switch (type) {
        case "challenge": return "#AA00AA"; // Purple
        case "goal": return "#55FFFF"; // Light blue
        case "root": return "#AAAAAA"; // Gray
        default: return "#55FF55"; // Green
    }
}

export function getFrameUrl(type, isCompleted) {
    let frame;
    switch (type) {
        case "challenge": frame = "challenge"; break;
        case "goal": frame = "goal"; break;
        default: frame = "task"; break;
    }
    const completed = isCompleted ? "_completed" : "";
    return `/frames/${frame}${completed}.png`;
}

export function formatDate(dateString) {
    if (!dateString) return "N/A";
    const date = new Date(dateString);
    if (isNaN(date.getTime())) return "Invalid Date";

    return date.toLocaleString(undefined, {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
    });
}