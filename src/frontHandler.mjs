export function initFront() {
    document.addEventListener("keydown", event => {
        if (
            event.key === "F5" ||
            (event.ctrlKey && event.key === "r") ||
            (event.metaKey && event.key === "r")
        ) {
            event.preventDefault();
        }
    });

    document.addEventListener("contextmenu", event => {
        event.preventDefault();
    });

    document.addEventListener("reload", event => {
        event.preventDefault();
    });

    window.addEventListener("DOMContentLoaded", () => {
        document.getElementById("t1").classList.add("load");
        document.getElementById("t2").classList.add("load");
        setTimeout(() => {
            document.getElementById("t2").classList.add("load2");
        }, 1000);
    });

    document.getElementById("logs").addEventListener("click", ev => {
        if (ev.target.classList.contains("chosen")) ev.preventDefault();
        document.getElementById("files").classList.remove("chosen");
        document.getElementById("live-stat").classList.add("logs");
        ev.target.classList.add("chosen");
    });
    document.getElementById("files").addEventListener("click", ev => {
        if (ev.target.classList.contains("chosen")) ev.preventDefault();
        document.getElementById("logs").classList.remove("chosen");
        document.getElementById("live-stat").classList.remove("logs");
        ev.target.classList.add("chosen");
    });
}