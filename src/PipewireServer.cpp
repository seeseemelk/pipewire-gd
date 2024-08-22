#include "PipewireServer.hpp"

#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/variant/utility_functions.hpp>

using namespace godot;

PipewireServer *PipewireServer::singleton = nullptr;

// Converts pipewire registry event details
// into a dictionary to be exposed in godot
static void on_registry_event(void *data, uint32_t id,
                uint32_t permissions, const char *type, uint32_t version,
                const struct spa_dict *props) {
	godot::UtilityFunctions::print("[pw] registry event received");

	Dictionary msg;
	msg["id"] = id;
	msg["type"] = type;
	msg["version"] = version;
	PipewireServer::get_singleton()->emit_signal(
		"registry_object_found",
		msg
	);

	PipewireServer::get_singleton()->sources.append(msg);
}

static const struct pw_registry_events registry_events = {
	PW_VERSION_REGISTRY_EVENTS,
	.global = on_registry_event
};

void PipewireServer::_bind_methods()
{
	//ClassDB::bind_signal("registry_object_found")
}

PipewireServer *PipewireServer::get_singleton()
{
	return singleton;
}

PipewireServer::PipewireServer()
{
	ERR_FAIL_COND(singleton != nullptr);

	godot::UtilityFunctions::print("[pw] connecting to pipewire");
	pw_init(NULL, NULL);

	Array sources;
	this->sources = sources;

	this->loop = pw_loop_new(NULL);

	ERR_FAIL_COND(this->loop == NULL);

	this->context = pw_context_new(this->loop,
				NULL /* properties */,
				0 /* user_data size */);

	this->core = pw_context_connect(this->context,
				NULL /* properties */,
				0 /* user_data size */);

	this->registry = pw_core_get_registry(this->core, PW_VERSION_REGISTRY,
				0 /* user_data size */);
	
	struct spa_hook registry_listener;
	spa_zero(registry_listener);

	pw_registry_add_listener(registry, &registry_listener, &registry_events, NULL);

	pw_loop_enter(this->loop);

	singleton = this;
}

void PipewireServer::_process(double delta) {
	godot::UtilityFunctions::print("[pw] POLL");
	int res = pw_loop_iterate(this->loop, -1);
	if (res < 0) {
		godot::UtilityFunctions::print("[pw] err when attempting loop", res);
		this->set_process(false);
	}
}

PipewireServer::~PipewireServer()
{
	ERR_FAIL_COND(singleton != this);
	singleton = nullptr;
	
	pw_loop_leave(this->loop);

	pw_proxy_destroy((struct pw_proxy*)this->registry);
	pw_core_disconnect(this->core);
	pw_context_destroy(this->context);
}
