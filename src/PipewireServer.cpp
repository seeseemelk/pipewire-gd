#include "PipewireServer.hpp"

#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/variant/utility_functions.hpp>

using namespace godot;

static const char* REGISTRY_OBJECT_ADD = "registry_object_found";
static const char* REGISTRY_OBJECT_REMOVE = "registry_object_lost";

typedef struct {
	uint32_t id;
} registry_event_remove_message;


// Converts pipewire registry event details
// into a dictionary to be exposed in godot
static void on_registry_event(void *_data, uint32_t id,
                uint32_t permissions, const char *type, uint32_t version,
                const struct spa_dict *props) {
	PipewireServer *server = (PipewireServer*)_data;

	Dictionary msg;
	msg["id"] = id;
	msg["type"] = type;
	msg["version"] = version;

	callable_mp(server, &PipewireServer::add_source).call_deferred(msg);
}

static void on_registry_remove_event(void *_data, uint32_t id) {
	PipewireServer *server = (PipewireServer*)_data;
	
	callable_mp(server, &PipewireServer::remove_source).call_deferred((int32_t)id);
}

// interrupts pipewire thread
static int do_stop(struct spa_loop *loop, bool async, uint32_t seq,
		const void *data, size_t size, void *_data)
{
	PipewireServer *server = (PipewireServer*)_data;
	server->running = false;
	return 0;
}


static const struct pw_registry_events registry_events = {
	PW_VERSION_REGISTRY_EVENTS,
	.global = on_registry_event,
	.global_remove = on_registry_remove_event,
};

void PipewireServer::_bind_methods()
{
	ADD_SIGNAL(MethodInfo(REGISTRY_OBJECT_ADD, PropertyInfo(Variant::DICTIONARY, "object")));
	ADD_SIGNAL(MethodInfo(REGISTRY_OBJECT_REMOVE, PropertyInfo(Variant::DICTIONARY, "object")));
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

PipewireServer::~PipewireServer()
{
	ERR_FAIL_COND(this->loop == nullptr);

	pw_proxy_destroy((struct pw_proxy*)this->registry);
	pw_core_disconnect(this->core);
	pw_context_destroy(this->context);
}

void PipewireServer::_enter_tree() {
	this->thread_loop.instantiate();
	this->thread_loop->start(callable_mp(this, &PipewireServer::poll_pw));
}

void PipewireServer::_exit_tree() {
	godot::UtilityFunctions::print("[pw] exiting loop", this->is_inside_tree());
	pw_loop_invoke(this->loop, do_stop, 1, NULL, 0, false, this);

	if (this->thread_loop->is_alive()) {
		this->thread_loop->wait_to_finish();
	}
	this->thread_loop->unreference();
}

void PipewireServer::poll_pw() {
	godot::UtilityFunctions::print("[pw] entering loop");

	this->running = true;
	
	pw_loop_enter(this->loop);
	while (this->running) {
		int res;
		if (res = pw_loop_iterate(this->loop, -1) < 0) {
			if (res != -EINTR) {
				break;
			}
		}
	}
	pw_loop_leave(this->loop);
}

Dictionary PipewireServer::get_sources() {
	return this->sources;
}

void PipewireServer::add_source(Dictionary source) {
	this->sources[source["id"]] = source;

	godot::UtilityFunctions::print("[pw] source: ", source);
	this->emit_signal(
		REGISTRY_OBJECT_ADD,
		source
	);
}

void PipewireServer::remove_source(int32_t id) {
	this->sources.erase(id);
	this->emit_signal(
		REGISTRY_OBJECT_REMOVE,
		id
	);
}
