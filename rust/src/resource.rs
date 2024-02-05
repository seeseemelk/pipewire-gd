use std::hash::Hash;

use godot::engine::notify::ObjectNotification;
use godot::prelude::*;
use godot::engine::{Image, ImageTexture, IImageTexture};

use crate::channels::*;

#[derive(GodotClass)]
#[class(tool, base=ImageTexture)]
pub struct PipewireTexture {
    #[export]
    source_id: u32,
    pub stream: Option<pipewire::stream::StreamListener<UserData>>,
    #[base]
    base: Base<ImageTexture>,
}

impl Eq for PipewireTexture { }

impl PartialEq for PipewireTexture {
    fn eq(&self, other: &Self) -> bool {
        return self.base().get_rid() == other.base().get_rid();
    }
}

impl Hash for PipewireTexture {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.base().get_rid().hash(state);
    }
}

#[godot_api]
impl PipewireTexture {
    #[func]
    fn update_from_stream(&mut self, source_id: u32, img: Gd<Image>) {
        if source_id != self.get_source_id() {
            return;
        }

        self.base_mut().update(img);
    }

    #[func]
    fn update_data_from_stream(&mut self, source_id: u32, data: PackedByteArray) {
        if source_id != self.get_source_id() {
            return;
        }

        let Some(mut image) = self.base_mut().get_image();
        image.set_data(
            image.get_width(),
            image.get_height(),
            image.has_mipmaps(),
            image.get_format(),
            data,
        );
    }
}

#[godot_api]
impl IImageTexture for PipewireTexture {
    fn on_notification(&mut self, what: ObjectNotification) {
        if what == ObjectNotification::Predelete {
            // TODO delete stream if no longer in use
        }
    }
}