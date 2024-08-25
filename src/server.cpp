#include "server.hpp"

#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/variant/utility_functions.hpp>

using namespace godot;

PipewireServer *PipewireServer::singleton = nullptr;

PipewireServer* PipewireServer::get_singleton()
{
	return singleton;
}

PipewireServer::PipewireServer()
{
	godot::UtilityFunctions::print("[pw] connecting to pipewire");
	pw_init(NULL, NULL);

	auto loop = this->create_loop();
	loop->prepare_for_registry();

	loop->connect(
		PW_SIGNAL,
		callable_mp(this, &PipewireServer::handle_event)
	);
	loop->start();

	this->registry_loop = loop;

	singleton = this;
}

PipewireServer::~PipewireServer()
{
	this->registry_loop->stop();
	
	singleton = nullptr;
}

void PipewireServer::handle_event(Dictionary event) {
	if (event["$type"] == PW_REGISTRY_ADD) {
		godot::UtilityFunctions::print("[pw] found source: ", event);
	}
	else if (event["$type"] == PW_REGISTRY_ADD) {
		godot::UtilityFunctions::print("[pw] removing source: ", event["id"]);
	}
}
