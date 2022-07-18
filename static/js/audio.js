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

    let state = [
        true,
        parseFloat(window.localStorage.getItem("volume") || 100) || 100
    ];

    const update = (p) => {
        // apply state delta
        state = p(state);

        // apply state to audio player
        state[1] = Math.max(0, Math.min(100, state[1]));
        window.localStorage.setItem("volume", state[1]);

        const appliedVolume = state[0] ? 0 : state[1];
        if (appliedVolume > 0) initialize();
        audio.volume = appliedVolume / 100;

        return state;
    };

    window.qaa = { update };
});