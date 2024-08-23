#pragma once

#include <godot_cpp/classes/node.hpp>
#include <godot_cpp/variant/dictionary.hpp>
#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/classes/engine.hpp>
#include <godot_cpp/classes/thread.hpp>
#include <pipewire/pipewire.h>

using namespace godot;

class PipewireServer : public Node
{
	GDCLASS(PipewireServer, Node);

private:
	Ref<Thread> thread_loop;
	struct pw_loop *loop;
	struct pw_core *core;
	struct pw_context *context;
	
	struct pw_registry *registry;
	struct spa_hook *registry_listener;

protected:
	static void _bind_methods();
	
public:
	Dictionary sources;
	bool running;

	PipewireServer();
	~PipewireServer();

	Dictionary get_sources(); 
	void _enter_tree() override;
	void _exit_tree() override;

	void poll_pw();

	void add_source(Dictionary source);
	void remove_source(int32_t source_id);
};
