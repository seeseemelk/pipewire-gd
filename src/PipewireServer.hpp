#pragma once

#include <godot_cpp/classes/node.hpp>
#include <godot_cpp/variant/array.hpp>
#include <godot_cpp/variant/dictionary.hpp>
#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/classes/engine.hpp>
#include <godot_cpp/classes/scene_tree.hpp>
#include <godot_cpp/classes/window.hpp>

#include <pipewire/pipewire.h>

using namespace godot;

class PipewireServer : public Node
{
	GDCLASS(PipewireServer, Node);

	static PipewireServer *singleton;

private:
	pw_loop *loop;
	pw_core *core;
	pw_context *context;
	pw_registry *registry;

protected:
	static void _bind_methods();
	
public:
	static PipewireServer *get_singleton();
	Array sources;

	PipewireServer();
	~PipewireServer();

	Array get_sources(); 
	void _process(double delta) override;

};
