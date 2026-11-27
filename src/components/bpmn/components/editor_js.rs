//! Aufrufe in `public/bpmn-editor.js`, das den Editor bpmn-js im Browser steuert.
//!
//! Nur im Browser vorhanden; auf dem Server sind die Funktionen leer, weil dort kein Editor läuft.

#[cfg(feature = "hydrate")]
mod browser {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = hbeBpmn, js_name = mount)]
        pub fn mount(
            container: &JsValue,
            xml: &str,
            on_change: &JsValue,
            on_save: &JsValue,
            on_error: &JsValue,
            on_simulation: &JsValue,
        );
        #[wasm_bindgen(js_namespace = hbeBpmn, js_name = destroy)]
        pub fn destroy();
        #[wasm_bindgen(js_namespace = hbeBpmn, js_name = markSaved)]
        pub fn mark_saved();
        #[wasm_bindgen(js_namespace = hbeBpmn, js_name = undo)]
        pub fn undo();
        #[wasm_bindgen(js_namespace = hbeBpmn, js_name = redo)]
        pub fn redo();
        #[wasm_bindgen(js_namespace = hbeBpmn, js_name = toggleSimulation)]
        pub fn toggle_simulation();
        #[wasm_bindgen(js_namespace = hbeBpmn, js_name = zoom)]
        pub fn zoom(step: i32);
        #[wasm_bindgen(js_namespace = hbeBpmn, js_name = importFile)]
        pub fn import_file();
        #[wasm_bindgen(js_namespace = hbeBpmn, js_name = exportXml)]
        pub fn export_xml(name: &str);
        #[wasm_bindgen(js_namespace = hbeBpmn, js_name = exportSvg)]
        pub fn export_svg(name: &str);
    }
}

#[cfg(feature = "hydrate")]
pub use browser::*;

#[cfg(not(feature = "hydrate"))]
mod server {
    pub fn destroy() {}
    pub fn mark_saved() {}
    pub fn undo() {}
    pub fn redo() {}
    pub fn toggle_simulation() {}
    pub fn zoom(_step: i32) {}
    pub fn import_file() {}
    pub fn export_xml(_name: &str) {}
    pub fn export_svg(_name: &str) {}
}

#[cfg(not(feature = "hydrate"))]
pub use server::*;
