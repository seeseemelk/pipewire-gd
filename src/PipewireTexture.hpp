#pragma once

#include <godot_cpp/classes/image_texture.hpp>
#include <godot_cpp/core/class_db.hpp>

using namespace godot;

class PipewireTexture : public ImageTexture
{
	GDCLASS(PipewireTexture, ImageTexture);

protected:
	static void _bind_methods();

public:
	PipewireTexture();
	~PipewireTexture();
};
