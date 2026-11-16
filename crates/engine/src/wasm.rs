use wasm_bindgen::prelude::*;

use crate::Engine;

#[wasm_bindgen]
pub struct Handle {
    engine: Engine,
}

#[wasm_bindgen]
impl Handle {
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: f32) -> Handle {
        Handle {
            engine: Engine::new(sample_rate),
        }
    }

    pub fn cmd(&mut self, json: &str) -> Result<(), JsValue> {
        self.engine.command_json(json).map_err(|err| JsValue::from_str(&err))
    }

    pub fn hud(&self) -> String {
        self.engine.hud_json()
    }

    pub fn catalog(&self) -> String {
        self.engine.catalog_json()
    }

    pub fn export_doc(&self) -> String {
        self.engine.export_json()
    }

    /// Interleaved stereo frames, left/right pairs, in wasm memory.
    pub fn render(&mut self, frames: usize) -> u32 {
        self.engine.render_interleaved(frames) as u32
    }
}

#[wasm_bindgen]
pub fn take_memory() -> JsValue {
    wasm_bindgen::memory()
}
