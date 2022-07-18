const init = () => {
    const events = new EventSource("/events");
    events.onmessage = ({ data }) => {
        const event = JSON.parse(data) || {};

        for(const [key, value] of Object.entries(event)) {
            window.qui.set(key, value);
        }
    };

    events.onerror = (e) => {
        events.close();
        setTimeout(init, 5000);
    };
};

$(init);