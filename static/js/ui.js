$(function() {
    //info
    const startAnimation = () => {
        const animatable = $(".animatable");
    
        animatable.addClass("anim");
        animatable.on("animationend", function() {
            $(this).removeClass("anim");
        })
    };

    const setBackground = (src) => {
        const curr = $(".background-image.current");
        const next = $(".background-image:not(.current)");
    
        next.attr("src", src);
        curr.removeClass("current");
        next.addClass("current");
    };

    const formatListeners = (l) => {
        switch(l) {
            case 0: return "No listeners";
            case 1: return "1 listener";
            default: 
                if (l < 1e3) return l + " listeners"; //as if people going to listen to this radio lol
                if (l >= 1e3 && l < 1e6) return +(l / 1e3).toFixed(1) + "k listeners"; 
                if (l >= 1e6 && l < 1e9) return +(l / 1e6).toFixed(1) + "m listeners";
                if (l >= 1e9 && l < 1e12) return +(l / 1e9).toFixed(1) + "b listeners";
                return +(l / 1e12).toFixed(1) + "t listeners";
        }
    }

    const update = ({ title, subtitle, author, background_url, source_url, listeners }) => {
        if(typeof title !== "undefined") $("#title").text(title || "");
        if(typeof subtitle !== "undefined") $("#subtitle").text(subtitle || "");
        if(typeof author !== "undefined") $("#author").text(author || "");
        if(title || subtitle || author) startAnimation();

        if(typeof listeners !== "undefined") $("#listeners").text(formatListeners(listeners));
        if(typeof background_url !== "undefined") setBackground(background_url);
    };

    (() => { //setting up the volume setting
        const elem = $("#volume");
        const btn = $("#volume-btn");
        let volume = 0;

        const updateVolume = (v) => {
            volume = v;
            elem.text(Math.floor(v));
        };

        btn.on("click", () => updateVolume(qaa.toggleMute()));
        btn.on("wheel", (e) => {
            updateVolume(qaa.setVolume(volume -= Math.sign(e.originalEvent.deltaY) * 5))
        });

        //event stream
        const events = new EventSource("/events");
        events.onmessage = ({ data }) => {
            const event = JSON.parse(data) || {};
            console.log(event)
            update(event);
        };
    })();

    window.qui = { update };
});