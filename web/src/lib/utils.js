export const CACHE_DIR = "caches"; // todo: env or something

export const DEFAULTS = {
    face: "/defaults/steve.png",
    world_icon: "/defaults/world-icon.png"
};


// get advancement icon
export function getIconUrl(icon) {
    if (!icon) {
        return "/icons/minecraft_barrier.png";
    }

    switch (icon.type) {
        case "playerHead":
            return icon.texture_id ? `/heads/${icon.texture_id}.png` : "/icons/minecraft_player_head.png";

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
    if (!dateString) return "";

    const date = new Date(dateString);
    const day = date.getDate().toString().padStart(2, "0");
    const month = (date.getMonth() + 1).toString().padStart(2, "0");
    const year = date.getFullYear().toString().slice(-2);
    const hours = date.getHours().toString().padStart(2, "0");
    const minutes = date.getMinutes().toString().padStart(2, "0");
    return `(Achieved ${day}/${month}/${year} ${hours}:${minutes})`;
}