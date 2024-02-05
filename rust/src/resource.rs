use godot::prelude::*;
use godot::engine::{Image, ImageTexture};

#[derive(GodotClass)]
#[class(tool, base=ImageTexture)]
pub struct PipewireTexture {
    #[base]
    base: Base<ImageTexture>,
}

#[godot_api]
impl PipewireTexture {
    #[func]
    fn update_from_stream(&mut self, img: Gd<Image>) {
        self.base_mut().update(img);
    }

    #[func]
    fn update_data_from_stream(&mut self, data: PackedByteArray) {
        let Some(mut image) = self.base_mut().get_image() else { return; };
        let width = image.get_width();
        let height = image.get_height();
        let mipmaps = image.has_mipmaps();
        let format = image.get_format();

        image.set_data(
            width,
            height,
            mipmaps,
            format,
            data,
        );
    }
}
