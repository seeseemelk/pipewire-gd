use std::collections::{HashSet, HashMap};
use std::hash::Hash;
use godot::builtin::{Array, GString};
use godot::engine::Image;
use godot::prelude::*;

use pipewire as pw;
use pw::stream::StreamListener;
use pw::{properties, spa};
use spa::pod::Pod;

use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;
use std::string::String;

use crate::channels::*;
use crate::resource::PipewireTexture;

struct Terminate;

struct PipewireSession {
    pw_thread: JoinHandle<()>,
    // messaging channels for killing the thread
    signal_terminate: pipewire::channel::Sender<Terminate>,
    res_sender: pipewire::channel::Sender<PipewireResourceNotify>,
    update_receiver: mpsc::Receiver<PipewireUpdateNotify>,
    available_sources: HashSet<u32>,
}

#[derive(GodotClass)]
#[class(tool, base=Node)]
pub struct PipewireClient {
    session: Option<PipewireSession>,

    #[base]
    base: Base<Node>,
}

// signals
const SIGNAL_PARAMETERS_CHANGED: StringName = StringName::from("frame_parameters_changed");
const SIGNAL_FRAME_UPDATE: StringName = StringName::from("frame_update");

#[godot_api]
impl INode for PipewireClient {
    fn init(base: Base<Node>) -> Self {
        Self {
            session: None,
            base,
        }
    }
}

#[godot_api]
impl PipewireClient {
    fn enter_tree(&mut self) {
        let (notify_update, update_receiver) = std::sync::mpsc::channel();
        let (kill_sender, kill_receiver) = pipewire::channel::channel();
        let (res_sender, res_receiver) = pipewire::channel::channel();

        // start up the thread once we're in the tree
        let pw_thread = thread::spawn(|| {
            // starts a pipewire main loop
            // we will manage this in a parallel thread to godot's main thread
            let Ok(mainloop) = pw::MainLoop::new() else { return; };
            let Ok(context) = pw::Context::new(&mainloop) else { return; };
            let Ok(core) = context.connect(None) else { return; };
            let Ok(registry) = core.get_registry() else { return; };

            let mut connected_sources = HashMap::new();

            // listen for video sources
            let _sourcesListener = registry
                .add_listener_local()
                .global(|global| {
                    godot_print!("source detected {:?}", global);

                    notify_update.send(PipewireUpdateNotify {
                        source_id: global.id,
                        image: None,
                        buffer: None,
                    });
                    // TODO when godot-rust bindings are better, emit a signal with source name/id
                })
                .register();

            // When we receive a `Terminate` message, quit the main loop.
            kill_receiver.attach(
                mainloop.as_ref(), 
                move |_| {
                    mainloop.quit();
                }
            );

            // create streams for already loaded textures
            res_receiver.attach(
                mainloop.as_ref(), 
                |notify: PipewireResourceNotify| {
                    match notify.action {
                        PipewireResourceNotifyAction::CREATED => {
                            let Some(source_id) = notify.source_id else { return; };
                            
                            // stream already exists
                            if let Some(_) = connected_sources.get(&source_id) {
                                return;
                            }

                            let Ok(listener) = Self::create_video_stream(
                                notify_update,
                                mainloop,
                                &core,
                                source_id,
                            ) else { return; };

                            connected_sources.insert(source_id, listener);
                        }
                        PipewireResourceNotifyAction::DELETED => {
                            let Some(source_id) = notify.source_id else { return; };
                            let Some(listener) = connected_sources.get(&source_id) else { return; };

                            connected_sources.remove(&source_id);

                            drop(listener);
                        }
                        _ => ()
                    }
                },
            );

            mainloop.run();
        });

        self.session = Some(PipewireSession {
            pw_thread,
            res_sender,
            signal_terminate: kill_sender,
            update_receiver,
            available_sources: HashSet::new(),
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

    fn process(&mut self, delta: f64) {
        let Some(session) = self.session else { return; };

        // read buffers from msq
        for msg in session.update_receiver {
            match msg {
                PipewireUpdateNotify {
                    source_id,
                    image: None,
                    buffer: None,
                } => {
                    session.available_sources.insert(
                        source_id,
                    );
                }
                PipewireUpdateNotify {
                    source_id,
                    image: Some(image),
                    buffer: None 
                } => {
                    let Some(img) = godot::engine::Image::create(
                        image.width,
                        image.height,
                        false,
                        godot::engine::image::Format::RGBA8,
                    );
    
                    // use godot channel to signal updates 
                    self.base().emit_signal(
                        SIGNAL_PARAMETERS_CHANGED,
                        &[source_id.to_variant(), img.to_variant()],
                    );
                }
                PipewireUpdateNotify {
                    source_id,
                    image: None,
                    buffer: Some(buffer)
                } => {
                    let gd_data = PackedByteArray::new();
                    gd_data.resize(buffer.size);
                    for i in 0..buffer.size {
                        gd_data.set(i, buffer.data[i]);
                    }

                    // use godot channel to signal updates 
                    self.base().emit_signal(
                        SIGNAL_FRAME_UPDATE,
                        &[source_id.to_variant(), gd_data.to_variant()],
                    );
                }
                _ => ()
            }
        }
    }

    #[func]
    fn enumerate(&mut self) -> Array<GString> {
        let mut arr = Array::new();
        
        let Some(session) = self.session else { return arr; };

        for global in session.available_sources {
            // arr.push(global.into());
        }
        return arr;
    }

    #[func]
    fn connect_texture(&mut self, texture: Gd<PipewireTexture>) {
        let Some(session) = self.session else { return; };
        let source_id = texture.bind().get_source_id();

        self.base().connect(
            SIGNAL_PARAMETERS_CHANGED,
            texture.callable("update_from_stream"),
        );
        self.base().connect(
            SIGNAL_FRAME_UPDATE,
            texture.callable("update_data_from_stream"),
        );

        session.res_sender.send(PipewireResourceNotify {
            action: PipewireResourceNotifyAction::CREATED,
            source_id: Some(source_id),
        });
    }

    // use this if it is desired to disconnect a texture from pipewire
    // before the texture resource as a whole is freed.
    // necessary to do if the texture is connected to a stream and you
    // wish to change which source it is streaming from
    #[func]
    fn disconnect_texture(&mut self, texture: Gd<PipewireTexture>) {
        let Some(session) = self.session else { return; };
        let source_id = texture.bind().get_source_id();
        
        session.res_sender.send(PipewireResourceNotify {
            action: PipewireResourceNotifyAction::DELETED,
            source_id: Some(source_id),
        });
    }

    fn create_video_stream(client: mpsc::Sender<PipewireUpdateNotify>, mainloop: pw::MainLoop, core: &pw::Core, source_id: u32) -> Result<pw::stream::StreamListener<UserData>, pw::Error>{
        let stream = pipewire::stream::Stream::new(
            core,
            "video-texture",
            properties!(
                *pw::keys::MEDIA_TYPE => "Video",
                *pw::keys::MEDIA_CATEGORY => "Capture",
                *pw::keys::MEDIA_ROLE => "Game",    
            )
        )?;

        let data = UserData {
            format: Default::default(),
        };

        let _listener = stream
            .add_local_listener_with_user_data(data)
            .state_changed(|old, new| {
                println!("State changed: {:?} -> {:?}", old, new);
            })
            .param_changed(move |_, id, user_data, param| {
                let Some(param) = param else {
                    return;
                };
                if id != libspa::param::ParamType::Format.as_raw() {
                    return;
                }

                let (media_type, media_subtype) =
                    match libspa::param::format_utils::parse_format(param) {
                        Ok(v) => v,
                        Err(_) => return,
                    };
 
                // only read from stream if it's video data
                if media_type != libspa::format::MediaType::Video
                    || media_subtype != libspa::format::MediaSubtype::Raw
                {
                    return;
                }

                user_data
                    .format
                    .parse(param)
                    .expect("Failed to parse param changed to VideoInfoRaw");

                let Ok(width) = user_data.format.size().width.try_into() else { return };
                let Ok(height) = user_data.format.size().height.try_into() else { return };
                
                client.send(PipewireUpdateNotify {
                    source_id,
                    image: Some(ImageParameters {
                        width,
                        height,
                        has_mipmaps: false,
                        format: godot::engine::image::Format::RGBA8,
                    }),
                    buffer: None
                });
            })
            .process(move |stream, _| {
                let Some( mut buffer) = stream.dequeue_buffer();
                let Some( mut data) =
                    if buffer.datas_mut().is_empty() { None }
                    else { Some(buffer.datas_mut()[0]) }
                ;
                
                // move frame data to godot byte array
                let Some(samples) = data.data();
                let chunk = data.chunk();
                let Ok(size) = chunk.size().try_into();
                let mut buffer = Vec::new();
                for i in 0..size {
                    buffer.push(samples[i]);
                }
                
                client.send(PipewireUpdateNotify {
                    source_id,
                    image: None,
                    buffer: Some(ImageBuffer {
                        data: buffer,
                        size,
                    }),
                });

            })
            .register()?;

        let obj = pw::spa::pod::object!(
            pw::spa::utils::SpaTypes::ObjectParamFormat,
            libspa::param::ParamType::EnumFormat,
            pw::spa::pod::property!(
                libspa::format::FormatProperties::MediaType,
                Id,
                libspa::format::MediaType::Video
            ),
            pw::spa::pod::property!(
                libspa::format::FormatProperties::MediaSubtype,
                Id,
                libspa::format::MediaSubtype::Raw
            ),
            pw::spa::pod::property!(
                libspa::format::FormatProperties::VideoFormat,
                Choice,
                Enum,
                Id,
                libspa::param::video::VideoFormat::RGB,
                libspa::param::video::VideoFormat::RGB,
                libspa::param::video::VideoFormat::RGBA,
                libspa::param::video::VideoFormat::RGBx,
                libspa::param::video::VideoFormat::BGRx,
                libspa::param::video::VideoFormat::YUY2,
                libspa::param::video::VideoFormat::I420,
            ),
            pw::spa::pod::property!(
                libspa::format::FormatProperties::VideoSize,
                Choice,
                Range,
                Rectangle,
                pw::spa::utils::Rectangle {
                    width: 320,
                    height: 240
                },
                pw::spa::utils::Rectangle {
                    width: 1,
                    height: 1
                },
                pw::spa::utils::Rectangle {
                    width: 4096,
                    height: 4096
                }
            ),
            pw::spa::pod::property!(
                libspa::format::FormatProperties::VideoFramerate,
                Choice,
                Range,
                Fraction,
                pw::spa::utils::Fraction { num: 25, denom: 1 },
                pw::spa::utils::Fraction { num: 0, denom: 1 },
                pw::spa::utils::Fraction {
                    num: 1000,
                    denom: 1
                }
            ),
        );
        let values: Vec<u8> = pw::spa::pod::serialize::PodSerializer::serialize(
            std::io::Cursor::new(Vec::new()),
            &pw::spa::pod::Value::Object(obj),
        )
            .unwrap()
            .0
            .into_inner();
        
        let mut params = [Pod::from_bytes(&values).unwrap()];

        stream.connect(
            libspa::Direction::Input,
            Some(source_id),
            pw::stream::StreamFlags::AUTOCONNECT | pw::stream::StreamFlags::MAP_BUFFERS,
            &mut params,
        )?;

        return Ok(_listener);
    }
}
