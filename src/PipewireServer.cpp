#include "PipewireServer.hpp"

#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/variant/utility_functions.hpp>

using namespace godot;

// Converts pipewire registry event details
// into a dictionary to be exposed in godot
static void on_registry_event(void *_data, uint32_t id,
                uint32_t permissions, const char *type, uint32_t version,
                const struct spa_dict *props) {
	PipewireServer *server = (PipewireServer*)_data;

	int64_t _id = id;
	Dictionary msg;
	msg["id"] = _id;
	msg["type"] = type;
	msg["version"] = version;

	server->emit_signal(
		"registry_object_found",
		msg
	);
	godot::UtilityFunctions::print("[pw] connecting to pipewire", msg);
	//server->sources[_id] = msg;
}

static void on_registry_remove_event(void *_data, uint32_t id) {
	PipewireServer *server = (PipewireServer*)_data;

	server->sources->erase(id);
}

static const struct pw_registry_events registry_events = {
	PW_VERSION_REGISTRY_EVENTS,
	.global = on_registry_event,
	.global_remove = on_registry_remove_event,
};

void PipewireServer::_bind_methods()
{
	ADD_SIGNAL(MethodInfo("registry_object_found", PropertyInfo(Variant::DICTIONARY, "object")));
	//ClassDB::bind_signal("registry_object_found")
}

Dictionary PipewireServer::get_sources() {
	return *(this->sources);
}

PipewireServer::PipewireServer()
{
	godot::UtilityFunctions::print("[pw] connecting to pipewire");
	pw_init(NULL, NULL);

	this->loop = pw_loop_new(NULL);

	this->context = pw_context_new(this->loop,
				NULL /* properties */,
				0 /* user_data size */);

	this->core = pw_context_connect(this->context,
				NULL /* properties */,
				0 /* user_data size */);

	this->registry = pw_core_get_registry(this->core, PW_VERSION_REGISTRY,
				0 /* user_data size */);
	
	ERR_FAIL_COND(this->registry == nullptr);

	this->registry_listener = new spa_hook;
	spa_zero(*(this->registry_listener));
	
	pw_registry_add_listener(this->registry, this->registry_listener, &registry_events, this);

	
	godot::UtilityFunctions::print("[pw] pipewire initialized");
}

void PipewireServer::_enter_tree() {
	godot::UtilityFunctions::print("[pw] entering loop");
	pw_loop_enter(this->loop);
}

void PipewireServer::_exit_tree() {
	godot::UtilityFunctions::print("[pw] exiting loop");
	pw_loop_leave(this->loop);
}

void PipewireServer::_process(double delta) {
	int res;
	ERR_FAIL_COND(this->loop == nullptr);

	// TODO look into using thread loop instead, since this blocks when waiting for new
	// messages from pipewire
	if (res = pw_loop_iterate(this->loop, -1) < 0) {
		// ignore interrupts
		if (res == -EINTR) {
			return;
		}
		godot::UtilityFunctions::print("[pw] err when attempting loop");
		this->set_process(false);
	}
	godot::UtilityFunctions::print("[pw] loop, res: ", res);
}

PipewireServer::~PipewireServer()
{
	ERR_FAIL_COND(this->loop == nullptr);
	
	pw_proxy_destroy((struct pw_proxy*)this->registry);
	pw_core_disconnect(this->core);
	pw_context_destroy(this->context);
}
