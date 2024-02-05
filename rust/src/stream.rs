use pipewire as pw;
use pw::{properties, spa};
use spa::pod::Pod;
use godot::prelude::*;

use crate::channels::UserData;
use crate::client::PipewireClient;
use crate::resource::PipewireTexture;

#[derive(GodotClass)]
#[class(tool, init, base=Node)]
pub struct PipewireStream {
    #[var]
    pub source_id: u32,
    listener: Option<pw::stream::StreamListener<UserData>>,

    #[base]
    base: Base<Node>
}

// signals
static SIGNAL_PARAMETERS_CHANGED: &str = "frame_parameters_changed";
static SIGNAL_FRAME_UPDATE: &str = "frame_update";

#[godot_api]
impl PipewireStream {
    #[func]
    fn connect_texture(&mut self, texture: Gd<PipewireTexture>) {
        self.base_mut().connect(
            StringName::from(SIGNAL_PARAMETERS_CHANGED),
            texture.callable("update_from_stream"),
        );
        self.base_mut().connect(
            StringName::from(SIGNAL_FRAME_UPDATE),
            texture.callable("update_data_from_stream"),
        );
    }
}

#[godot_api]
impl INode for PipewireStream {
    
    fn enter_tree(&mut self) {
        let Some(parent_node) = self.base().get_parent() else { return; };
        let Ok(client) = parent_node.try_cast::<PipewireClient>() else { return; };
        let pw_core = &client.bind().pw_core;

        let Ok(stream) = pipewire::stream::Stream::new(
            pw_core,
            "video-texture",
            properties!(
                *pw::keys::MEDIA_TYPE => "Video",
                *pw::keys::MEDIA_CATEGORY => "Capture",
                *pw::keys::MEDIA_ROLE => "Game",    
            )
        ) else { return; };

        let data = UserData {
            format: Default::default(),
        };

        let Ok(_listener) = stream
            .add_local_listener_with_user_data(data)
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
                
                let img = godot::engine::Image::create(
                    width, height, false,
                    godot::engine::image::Format::RGBA8
                );

                self.base_mut().emit_signal(
                    StringName::from(SIGNAL_PARAMETERS_CHANGED),
                    &[img.to_variant()],
                );
            })
            .process(move |stream, _| {
                let Some( mut buffer) = stream.dequeue_buffer() else { return; };
                if buffer.datas_mut().is_empty() {
                    return;
                }

                let Some(data) = buffer.datas_mut().get(0) else { return; };
                
                // move frame data to godot byte array
                let Some(samples) = data.data() else { return; };
                let chunk = data.chunk();
                let Ok(size) = chunk.size().try_into() else { return; };
                let mut buffer = PackedByteArray::new();
                buffer.resize(size);
                for i in 0..size {
                    buffer.set(i, samples[i]);
                }
                
                self.base_mut().emit_signal(
                    StringName::from(SIGNAL_FRAME_UPDATE),
                    &[buffer.to_variant()],
                );

            })
            .register() else { return; };

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
            Some(self.source_id),
            pw::stream::StreamFlags::AUTOCONNECT | pw::stream::StreamFlags::MAP_BUFFERS,
            &mut params,
        );

        self.listener = Some(_listener);
    }

    fn exit_tree(&mut self) {
        let Some(listener) = &self.listener else { return; };

        drop(listener);
        self.listener = None;
    }

    fn process(&mut self, _delta: f64) {

    }
}