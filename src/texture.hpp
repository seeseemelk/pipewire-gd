#pragma once

#include <godot_cpp/classes/image_texture.hpp>
#include <godot_cpp/classes/engine.hpp>
#include <godot_cpp/core/class_db.hpp>
#include <pipewire/pipewire.h>

#include "loop.hpp"
#include "server.hpp"

using namespace godot;

class PipewireTexture : public ImageTexture
{
	GDCLASS(PipewireTexture, ImageTexture);

private:
	char* path;
	Ref<PipewireLoop> loop;

protected:
	static void _bind_methods();

public:
	PipewireTexture();
	~PipewireTexture();

	void _initialize_pw_stream();
};
