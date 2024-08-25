#include "texture.hpp"

#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/variant/utility_functions.hpp>
#include <godot_cpp/classes/rendering_server.hpp>

#include <spa/param/video/format-utils.h>
#include <spa/debug/types.h>
#include <spa/param/video/type-info.h>

using namespace godot;

void PipewireTexture::_bind_methods()
{
}

void PipewireTexture::_initialize_pw_stream() {
    auto props = pw_properties_new(PW_KEY_MEDIA_TYPE, "Video",
        PW_KEY_MEDIA_CATEGORY, "Capture",
        PW_KEY_MEDIA_ROLE, "Camera",
        PW_KEY_PRIORITY_DRIVER, "10000",
        NULL);
    
    Ref<PipewireLoop> loop = PipewireServer::get_singleton()->create_loop();
    
    loop->prepare_stream(
        this->path,
        props
    );
}

PipewireTexture::PipewireTexture()
{
    
}

PipewireTexture::~PipewireTexture()
{
    
}
