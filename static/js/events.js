const init = () => {
    const events = new EventSource("/events");

    events.addEventListener("message", ({ data }) => {
        const event = JSON.parse(data) || {};

        for(const [key, value] of Object.entries(event)) {
            window.qui.set(key, value);
        }
    });

    events.addEventListener("error", () => {
        events.close();
        setTimeout(init, 5000);
    });
};

$(init);


// keep-alive
$(() => {
    setInterval(() => {
        const request = new XMLHttpRequest();
        request.open("GET", "/status", true);
        request.send(null);
    }, 60 * 10 * 1000);
});