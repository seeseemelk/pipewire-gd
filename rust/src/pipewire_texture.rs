use std::collections::HashSet;
use std::hash::Hash;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use pipewire::channel::{Sender as PSender, Receiver as PReceiver};

use godot::engine::notify::ObjectNotification;
use godot::prelude::*;
use godot::engine::{Engine, Image, ImageTexture, IImageTexture};

pub const PIPEWIRE_TEXTURES: HashSet<PipewireTexture> = HashSet::new();

// used to notify when new resources are allocated
pub const RESOURCE_CHANNEL: (Sender<PipewireResourceNotify>, Receiver<PipewireResourceNotify>) = mpsc::channel();

// use to pass pipewire textures back to the associated resources
// so that they may update their images
pub const BUFFER_CHANNEL: (PSender<PipewireBuffer>, PReceiver<PipewireBuffer>) = pipewire::channel::channel();

enum PipewireResourceNotifyAction {
    CREATED,
    DELETED
}

pub struct PipewireResourceNotify {
    action: PipewireResourceNotifyAction,
    resource: PipewireTexture,
}

pub struct PipewireBuffer {
    rid: Rid,
    buffer: Image,
}

#[derive(GodotClass)]
#[class(tool, base=ImageTexture)]
pub struct PipewireTexture {
    #[export]
    source: GString,
    #[base]
    base: Base<ImageTexture>
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
impl IImageTexture for PipewireTexture {
    fn init(base: Base<ImageTexture>) -> Self {
        let pw = Engine::singleton().get_singleton(StringName::from("Pipewire"));

        let s = Self {
            source: GString::from(""),
            base,
        };

        let rid = s.base().get_rid();

        PIPEWIRE_TEXTURES.insert(s);
        RESOURCE_CHANNEL.0.send( PipewireResourceNotify {
            action: PipewireResourceNotifyAction::CREATED,
            resource: s
        });

        return s;
    }

    fn on_notification(&mut self, what: ObjectNotification) {
        if what == ObjectNotification::Predelete {
            let rid = self.base().get_rid();

            PIPEWIRE_TEXTURES.remove(self);

            RESOURCE_CHANNEL.0.send( PipewireResourceNotify {
                action: PipewireResourceNotifyAction::DELETED,
                resource: *self
            });
        }
    }
}