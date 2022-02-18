$(function() {
    const audio = new Audio();
    audio.src = "/stream?r=" + Math.random().toString(36).slice(2);
    audio.crossOrigin = "anonymous";
    audio.preload = "none";
    audio.volume = 0;

    let initialize = () => {
        initialize = () => {};

        audio.load();
        audio.addEventListener("canplay", () => audio.play());
        audio.addEventListener("paused", () => audio.load());
    };

    let isMuted = true;
    let volume = parseFloat(window.localStorage.getItem("volume") || 100) || 100;

    const setGain = (g) => {
        initialize();
        let volume = Math.max(0, Math.min(100, g));
        audio.volume = volume / 100;
        return volume;
    }

    window.qaa = {
        setVolume: (vol) => {
            if (isMuted) {
                return 0;
            }

            isMuted = false;
            volume = vol;

            window.localStorage.setItem("volume", volume);
            return setGain(volume);
        },

        toggleMute: () => {
            isMuted = !isMuted;
            return setGain(isMuted ? 0 : volume);
        }
    }
});