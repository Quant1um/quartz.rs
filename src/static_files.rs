
use rocket::{routes, get, Route};
use rocket::response::content::*;

macro_rules! import {
    ($id:ident, $ty:ident, $path:literal) => { import!($id, $ty, $path, $path); };

    ($id:ident, $ty:ident, $path:literal, $bind:literal) => {
        #[get($bind)]
        fn $id() -> $ty<&'static [u8]> {
            $ty(include_bytes!(concat!("../static", $path)))
        }
    };
}

import!(index,      Html,       "/index.html", "/");
import!(js_jquery,  JavaScript, "/js/jquery.min.js");
import!(js_audio,   JavaScript, "/js/audio.js");
import!(js_ui,      JavaScript, "/js/ui.js");
import!(css_norm,   Css,        "/css/normalize.css");
import!(css_style,  Css,        "/css/style.css");

pub fn routes() -> Vec<Route> {
    routes![
        index,
        js_jquery,
        js_audio,
        js_ui,
        css_norm,
        css_style
    ]
}