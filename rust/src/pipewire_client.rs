use godot::builtin::{Array, GString};
use godot::engine::{RenderingServer};
use godot::prelude::*;
use pipewire::channel::AttachedReceiver;
use pipewire::constants::ID_ANY;
use libspa::pod::builder::builder_add;
use libspa::pod::{object, property};
use pipewire::spa::pod::Pod;
use pipewire::stream::StreamFlags;
use pipewire::{properties, spa, Core};
use pipewire::{MainLoop, Context, Stream};
use pipewire::properties::{Properties};
use core::any::Any;
use std::hash::Hash;
use std::ptr::{null, null_mut};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::thread::Thread;
use std::fmt::format;
use std::string::String;
use std::collections::HashSet;

use crate::pipewire_texture::{PipewireTexture, BUFFER_CHANNEL, PIPEWIRE_TEXTURES, RESOURCE_CHANNEL};

struct Terminate;

struct PipewireSession {
    pw_thread: JoinHandle<()>,
    // messaging channels for killing the thread
    signal_terminate: pipewire::channel::Sender<Terminate>
}

#[derive(GodotClass)]
#[class(tool, init, base=Object)]
pub struct PipewireClient {
    session: Option<PipewireSession>,
    available_sources: HashSet<String>,

    #[base]
    base: Base<Object>,
}

#[godot_api]
impl PipewireClient {
    fn init(base: Base<Object>) -> Self {
        Self {
            session: None,
            available_sources: HashSet::new(),
            base,
        }
    }

    fn enter_tree(&mut self) {
        let (main_sender, main_receiver) = mpsc::channel();
        let (pw_sender, pw_receiver) = pipewire::channel::channel();

        self.available_sources.clear();

        // start up the thread once we're in the tree 
        let pw_thread = thread::spawn(|| {
            // starts a pipewire main loop
            // we will manage this in a parallel thread to godot's main thread
            let mainloop = MainLoop::new()?;
            let context = Context::new(&mainloop)?;
            let core = context.connect(None)?;
            let registry = core.get_registry()?;

            let _sourcesListener = registry
                .add_listener_local()
                .global(|global| {
                    self.available_sources.insert(
                        format!("{:?}", global)
                    );
                    godot_print!("source added {:?}", global);
                    // TODO when godot-rust bindings are better, emit a signal with source name/id
                })
                .register();

            // When we receive a `Terminate` message, quit the main loop.
            let kill_receiver = pw_receiver.attach(
                mainloop.loop_(), 
                {
                    let mainloop = mainloop.clone();
                    move |_| mainloop.quit()
                }
            );

            // create streams for already loaded textures
            for texture in &PIPEWIRE_TEXTURES {
                Self::create_stream(
                    mainloop,
                    &core,
                    texture,
                );
            }

            // attach listener to create streams when a new texture is referenced

            mainloop.run();
        });

        self.session = Some(PipewireSession {
            pw_thread,
            signal_terminate: pw_sender
        });
    }

    fn exit_tree(&mut self) {
        // kill the thread
        if let Some(s) = self.session {
            s.signal_terminate.send(Terminate);
            s.pw_thread.join();
            
            self.session = None
        }
    }

    #[func]
    fn enumerate(&self) -> Array<GString> {
        let mut arr = Array::new();
        for global in &self.available_sources {
            arr.push(global.into());
        }
        return arr;
    }

    fn create_stream(mainloop: MainLoop, core: &Core, source: &PipewireTexture) -> Result<(), pipewire::Error>{
        let stream = pipewire::stream::Stream::new(
            core,
            &source.base().get_rid().to_string(),
            properties!(
                "media.type" => "Video",
                "media.category" => "Capture",
                "media.role" => "Game",
            )
        )?;

        let data: Vec<u8> = Vec::new();
        let builder = libspa::pod::builder::Builder::new(&mut data);
        let obj = object!(
            libspa::utils::SpaTypes::ObjectParamFormat,
            libspa::param::ParamType::EnumFormat,
            property!(
                libspa::param::format::FormatProperties::MediaType,
                Id,
                libspa::param::format::MediaType::Video
            ),
            property!(
                libspa::param::format::FormatProperties::MediaSubtype,
                Id,
                libspa::param::format::MediaSubtype::Raw
            ),
        );
        builder.push_object(obj.properties, obj.type_, obj.id)?;
        let pod: Pod = Pod::builder.state().into();

        stream.connect(
            spa::Direction::Input,
            Some(ID_ANY),
            StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS,
            &mut pod,
        );

        return Ok(());
    }
}
