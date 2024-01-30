use godot::prelude::*;
use godot::engine::Engine;

mod pipewire_client;
mod pipewire_texture;

use crate::pipewire_client::PipewireClient;
use crate::pipewire_texture::PipewireTexture;

struct PipewireExtension;

#[gdextension]
unsafe impl ExtensionLibrary for PipewireExtension {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            // The StringName identifies your singleton and can be
            // used later to access it.
            Engine::singleton().register_singleton(
                StringName::from("Pipewire"),
                PipewireClient::new_alloc().upcast(),
            );
        }
    }

    fn on_level_deinit(level: InitLevel) {
        if level == InitLevel::Scene {
            // Unregistering is needed to avoid memory leaks and 
            // warnings, especially for hot reloading.
            Engine::singleton().unregister_singleton(
                StringName::from("Pipewire")
            );
        }
    }
}