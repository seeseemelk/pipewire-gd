#pragma once

#include <godot_cpp/classes/object.hpp>
#include <godot_cpp/classes/ref.hpp>
#include <godot_cpp/classes/engine.hpp>
#include <godot_cpp/variant/dictionary.hpp>
#include <godot_cpp/core/class_db.hpp>
#include <pipewire/pipewire.h>
#include <map>

#include "loop.hpp"

using namespace godot;

/**
 * Singleton manager for Pipewire in Godot.
 * Acts as a factory for pipewire threads.
 */
class PipewireServer : public Object
{
	GDCLASS(PipewireServer, Object);

private:
	static PipewireServer *singleton;

	Ref<PipewireLoop> registry_loop;

	/* handles registry events from pipewire */
	void handle_event(Dictionary);

protected:
	static void _bind_methods() {};

public:
	static PipewireServer *get_singleton();

	PipewireServer();
	~PipewireServer();

	Ref<PipewireLoop> create_loop();
};
