#pragma once

#include <godot_cpp/classes/ref_counted.hpp>
#include <godot_cpp/classes/ref.hpp>
#include <godot_cpp/classes/thread.hpp>
#include <godot_cpp/variant/dictionary.hpp>
#include <godot_cpp/core/class_db.hpp>

#include <spa/param/video/format-utils.h>
#include <spa/debug/types.h>
#include <spa/param/video/type-info.h>
#include <pipewire/pipewire.h>

using namespace godot;

const char* PW_SIGNAL = "pw_event";
const char* PW_REGISTRY_ADD = "registry:add_source";
const char* PW_REGISTRY_REMOVE = "registry:remove_source";

/**
 * Pipewire requires a thread/loop per source that it's listening to
 */
class PipewireLoop : public RefCounted {
	GDCLASS(PipewireLoop, RefCounted);

	bool running;

private:	
	Ref<Thread> thread_loop;
	struct pw_properties *props;
	struct pw_loop *loop;
	
    /* registry */
	struct pw_core *core;
	struct pw_context *context;
	struct pw_registry *registry;
	struct spa_hook *registry_listener;

    /* video stream */
	struct pw_stream *stream;
    struct spa_video_info format;

    /* thread task polling pipewire */
	void poll_pw();

protected:
	static void _bind_methods() {
        ADD_SIGNAL(MethodInfo(PW_SIGNAL, PropertyInfo(Variant::DICTIONARY, "object")));
    };
	
public:
	pw_loop* get_loop();
	void start();
    void prepare_for_registry();
    void prepare_stream(char* path, pw_properties *props);
    void stop();
    void _thread_stop();

    /** generic event handler for pipewire
     *  used whenever we need to bridge the thread context with Godot
     */
    void handle_event(Dictionary msg);

	PipewireLoop();
	~PipewireLoop();
};
