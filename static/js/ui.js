$(function() {
    //info
    const startAnimation = () => {
        const animatable = $(".animation:not(.anim-playing)");

        animatable.addClass("anim-playing");
        animatable.removeClass("anim-init");
        animatable.one("animationend", function() {
            $(this).removeClass("anim-playing");
        });
    };

    const set = (key, value) => {
        switch (key) {
            case "title":
                $("#title").text(value || "");
                startAnimation();
                break;
            case "subtitle":
                $("#subtitle").text(value || "");
                startAnimation();
                break;
            case "author":
                $("#author").text(value || "");
                startAnimation();
                break;

            case "listeners":
            {
                switch (value) {
                    case 0:
                        $("#listeners").text("No listeners");
                        break;
                    case 1:
                        $("#listeners").text("1 listener");
                        break;
                    default:
                        let text;

                        if (value >= 1e9) text = (value / 1e9).toFixed(1) + "b listeners"; //LOL
                        else if (value >= 1e6) text = (value / 1e6).toFixed(1) + "m listeners"; //lol
                        else if (value >= 1e3) text = (value / 1e3).toFixed(1) + "k listeners";
                        else text = value + " listeners";

                        $("#listeners").text(text);
                        break;
                }

                break;
            }

            case "source_url":
            {
                $("#source").attr("href", value);

                if (value === null) {
                    $("#source").addClass("data-link-disabled");
                } else {
                    $("#source").removeClass("data-link-disabled");
                }

                break;
            }

            case "background_url":
            {
                //thubnail
                $("#thumbnail").css("background-image", `url(${value})`)

                //background
                const curr = $(".background-image.current");
                const next = $(".background-image:not(.current)");

                next.attr("src", value);
                curr.removeClass("current");
                next.addClass("current");

                startAnimation();
                break;
            }
        }
    };

    (() => {
        const thumbnail = $("#thumbnail");
        const volumeSlider = $("#volume-slider");
        const volumeHandle = $("#volume-handle");

        const updateState = ([paused, volume]) => {
            if (paused) {
                thumbnail.addClass("paused");
            } else {
                thumbnail.removeClass("paused");
            }

            volumeHandle.css("width", `${volume}%`);
        };

        thumbnail.on("click", () => {
            updateState(window.qaa.update(([paused, volume]) => [!paused, volume]));
        });

        thumbnail.on("wheel", (e) => {
            const delta = -2.5 * Math.sign(e.originalEvent.deltaY);
            updateState(window.qaa.update(([paused, volume]) => [paused, volume + delta]));
            e.preventDefault();
        });

        const processVolumeSlider = (e) => {
            e.preventDefault();

            const rect = e.currentTarget.getBoundingClientRect();
            const x = (e.clientX - rect.left) / rect.width;

            console.log(x);

            updateState(window.qaa.update(([paused, volume]) => [paused, x * 100]));
        };

        volumeSlider.on("click", (e) => processVolumeSlider);
    })();

    window.qui = { set };
});