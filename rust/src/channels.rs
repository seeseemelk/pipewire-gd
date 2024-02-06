use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use godot::builtin::PackedByteArray;
use godot::prelude::Gd;
use godot::engine::Image;

// used to notify when new resources are allocated
pub enum PipewireResourceNotifyAction {
    CREATED,
    DELETED,
    CLOSE
}

pub struct ImageParameters {
    pub width: i32,
    pub height: i32,
    pub has_mipmaps: bool,
    pub format: godot::engine::image::Format
}

pub struct ImageBuffer {
    pub samples: Vec<u8>,
    pub size: usize,
}

pub struct PipewireUpdateNotify {
    pub source_id: u32,
    pub image: Option<ImageParameters>,
    pub buffer: Option<ImageBuffer>
}

pub struct PipewireResourceNotify {
    pub action: PipewireResourceNotifyAction,
    pub source_id: Option<u32>,
}
