(function () {
    if (typeof window.initialized !== 'undefined') return;
    window.initialized = true;

    const LOUDNESS_MULTIPLIER = 3;
    const SEEK_SECONDS = 3;

    window.currentFile = null;

    let player = document.getElementById("player");
    let autoplayToggle = document.getElementById("autoplayToggle");

    function setupElementListeners() {
        autoplayToggle.addEventListener("click", (e) => {
            e.preventDefault();

            autoplayToggle.dataset["value"] = autoplayToggle.dataset["value"] === "true" ? "false" : "true";

            autoplayToggle.innerText = autoplayToggle.dataset["value"] === "true" ? "Autoplay ON" : "Autoplay OFF";
        });

        player.addEventListener("ended", () => {
            if (autoplayToggle.dataset["value"] !== "true") return;

            selectRelativeFile(1);
        });
    }

    htmx.on('htmx:historyRestore', function () {
        player = document.getElementById("player");
        autoplayToggle = document.getElementById("autoplayToggle");
        setupElementListeners();
    });

    if (navigator && navigator.audioSession)
        navigator.audioSession.type = "playback";

    window.setAudioSource = function(e) {
        player.parentElement.style.display = "block";
        player.src = '/file/' + e.target.dataset["path"];
        player.play();
        context.resume();
        window.currentFile = e.target.dataset["path"];
        document.querySelectorAll("a[data-active]").forEach(el => delete el.dataset["active"]);
    }

    document.getElementById("fileListing").addEventListener('htmx:load', function() {
        if (window.currentFile === null) return;
        let el = document.querySelector("a[data-path=\"" + window.currentFile + "\"]");
        if (el === null) return;
        el.dataset["active"] = "true";
    });

    function getFileEls() {
        return Array.from(document.querySelectorAll("#fileListing a[data-path]"));
    }

    function changeAudio(el) {
        el.scrollIntoView({
            block: "nearest",
            behavior: "smooth"
        });
        document.querySelectorAll("#fileListing a[data-active]").forEach(el => delete el.dataset["active"])
        el.click();
    }

    function selectRelativeFile(deltaIndex) {
        const files = getFileEls();
        const activeIndex = files.findIndex(f => f.hasAttribute("data-active"));
        const newIndex = activeIndex + deltaIndex;

        if (newIndex < 0 || newIndex >= files.length) return;

        changeAudio(files[activeIndex + deltaIndex]);
    }

    document.addEventListener("keydown", (e) => {
        if (e.shiftKey || e.altKey || e.ctrlKey || e.metaKey) return;
        if (e.key === "ArrowDown" || e.key === "ArrowUp") {
            e.preventDefault();

            if (e.key === "ArrowDown") {
                selectRelativeFile(1);
            } else if (e.key === "ArrowUp") {
                selectRelativeFile(-1);
            }
        } else if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
            e.preventDefault();
            if (player === null || player.currentSrc === "") return;
            if (e.key === "ArrowLeft") {
                player.currentTime = player.currentTime - SEEK_SECONDS;
                if (player.paused) {
                    player.play();
                    context.resume();
                }
            } else if (e.key === "ArrowRight") {
                player.currentTime = player.currentTime + SEEK_SECONDS;
            }
        } else if (e.key === " ") {
            e.preventDefault();

            if (player === null || player.currentSrc === "") return;

            if (player.paused) {
                player.play();
                context.resume();
            } else {
                player.pause();
            }
        } else if (e.key === "a") {
            e.preventDefault();
            autoplayToggle.click();
        }
    });

    const context = new AudioContext();
    const source = context.createMediaElementSource(player);
    const gain = context.createGain();
    source.connect(gain);
    gain.connect(context.destination);
    gain.gain.value = LOUDNESS_MULTIPLIER;

    setupElementListeners();
})();
