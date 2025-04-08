#include "loop.hpp"

#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/variant/utility_functions.hpp>

using namespace godot;

#pragma region Registry
// Converts pipewire registry event details
// into a dictionary to be exposed in godot
static void on_registry_event(void *_data, uint32_t id,
                uint32_t permissions, const char *type, uint32_t version,
                const struct spa_dict *props) {
	PipewireLoop *lp = (PipewireLoop*)_data;

	Dictionary msg;
	msg["$type"] = PW_REGISTRY_ADD;
	msg["id"] = (int32_t)id;
	msg["type"] = type;
	msg["version"] = (int32_t)version;

	callable_mp(lp, &PipewireLoop::handle_event).call_deferred(msg);
}

static void on_registry_remove_event(void *_data, uint32_t id) {
	PipewireLoop *lp = (PipewireLoop*)_data;
	
	Dictionary msg;
	msg["$type"] = PW_REGISTRY_REMOVE;
	msg["id"] = (int32_t)id;

	callable_mp(lp, &PipewireLoop::handle_event).call_deferred(msg);
}

static const struct pw_registry_events registry_events = {
	PW_VERSION_REGISTRY_EVENTS,
	.global = on_registry_event,
	.global_remove = on_registry_remove_event,
};

/** Adds controls to this loop so it'll be ready to listen to events from the
 *  pipewire registry during execution.
 */
void PipewireLoop::prepare_for_registry() {
    // can not add new contexts to a loop after we've already started processing from it
    ERR_FAIL_COND(this->running == true);

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
}
#pragma endregion Registry

#pragma region Stream
typedef struct {
    PipewireLoop *loop;
    pw_stream *stream;
    spa_video_info format;
} StreamReference;

static void on_stream_process(void *userdata)
{
        StreamReference *data = (StreamReference*)userdata;
        struct pw_buffer *b;
        struct spa_buffer *buf;
 
        if ((b = pw_stream_dequeue_buffer(data->stream)) == NULL) {
                pw_log_warn("out of buffers: %m");
                return;
        }
 
        buf = b->buffer;
        if (buf->datas[0].data == NULL)
                return;
 
        printf("got a frame of size %d\n", buf->datas[0].chunk->size);
 
        pw_stream_queue_buffer(data->stream, b);
}

static void on_param_changed(void *userdata, uint32_t id, const struct spa_pod *param)
{
        StreamReference *data = (StreamReference*)userdata;
 
        if (param == NULL || id != SPA_PARAM_Format)
                return;
 
        if (spa_format_parse(param,
                        &data->format.media_type,
                        &data->format.media_subtype) < 0)
                return;
 
        if (data->format.media_type != SPA_MEDIA_TYPE_video ||
            data->format.media_subtype != SPA_MEDIA_SUBTYPE_raw)
                return;
 
        if (spa_format_video_raw_parse(param, &data->format.info.raw) < 0)
                return;
 
        printf("got video format:\n");
        printf("  format: %d (%s)\n", data->format.info.raw.format,
                        spa_debug_type_find_name(spa_type_video_format,
                                data->format.info.raw.format));
        printf("  size: %dx%d\n", data->format.info.raw.size.width,
                        data->format.info.raw.size.height);
        printf("  framerate: %d/%d\n", data->format.info.raw.framerate.num,
                        data->format.info.raw.framerate.denom);
 
}

static const struct pw_stream_events stream_events = {
        PW_VERSION_STREAM_EVENTS,
        .param_changed = on_param_changed,
        .process = on_stream_process,
};

/** Attaches a capture stream to the loop, mainly for listening to video
 *  playback that can be fed to Godot textures
 */
void PipewireLoop::prepare_stream(char* path, pw_properties *props) {
    // can not add new contexts to a loop after we've already started processing from it
    ERR_FAIL_COND(this->running == true);

    pw_properties_set(props, PW_KEY_TARGET_OBJECT, path);

    auto data = StreamReference {
        .loop = this, 
    };
    auto stream = pw_stream_new_simple(
        loop,
        "video-play",
        props,
        &stream_events,
        &data
    );
    data.stream = stream;
    
    uint8_t buffer[1024];
    spa_pod_builder b = SPA_POD_BUILDER_INIT(buffer, sizeof(buffer));
    const spa_pod *params[1];

    auto rect240 = SPA_RECTANGLE(320, 240);
    auto rect1 = SPA_RECTANGLE(1, 1);
    auto rect4096 = SPA_RECTANGLE(4096, 4096);

    auto fract25 = SPA_FRACTION(25, 1);
    auto fract0 = SPA_FRACTION(0, 1);
    auto fract1000 = SPA_FRACTION(1000, 1);

    auto p = spa_pod_builder_add_object(&b,
        SPA_TYPE_OBJECT_Format, SPA_PARAM_EnumFormat,
        SPA_FORMAT_mediaType,       SPA_POD_Id(SPA_MEDIA_TYPE_video),
        SPA_FORMAT_mediaSubtype,    SPA_POD_Id(SPA_MEDIA_SUBTYPE_raw),
        SPA_FORMAT_VIDEO_format,    SPA_POD_CHOICE_ENUM_Id(7,
                                        SPA_VIDEO_FORMAT_RGB,
                                        SPA_VIDEO_FORMAT_RGB,
                                        SPA_VIDEO_FORMAT_RGBA,
                                        SPA_VIDEO_FORMAT_RGBx,
                                        SPA_VIDEO_FORMAT_BGRx,
                                        SPA_VIDEO_FORMAT_YUY2,
                                        SPA_VIDEO_FORMAT_I420),
        SPA_FORMAT_VIDEO_size,      SPA_POD_CHOICE_RANGE_Rectangle(
                                        &rect240,
                                        &rect1,
                                        &rect4096),
        SPA_FORMAT_VIDEO_framerate, SPA_POD_CHOICE_RANGE_Fraction(
                                        &fract25,
                                        &fract0,
                                        &fract1000)
    );
    params[0] = (spa_pod*)p;

    int res = pw_stream_connect(stream,
                        PW_DIRECTION_INPUT,
                        PW_ID_ANY,
                        static_cast<pw_stream_flags>(
                            PW_STREAM_FLAG_AUTOCONNECT |
                            PW_STREAM_FLAG_MAP_BUFFERS
                        ),
                        params, 1);
                        
    ERR_FAIL_COND(res < 0);

    this->stream = stream;
}
#pragma endregion Stream

// interrupts pipewire thread
static int do_stop(struct spa_loop *loop, bool async, uint32_t seq,
		const void *data, size_t size, void *_data)
{
	PipewireLoop *lp = (PipewireLoop*)_data;
	lp->_thread_stop();
	return 0;
}


pw_loop* PipewireLoop::get_loop() {
	return this->loop;
}

void PipewireLoop::poll_pw() {
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

void PipewireLoop::start() {
    godot::UtilityFunctions::print("[pw] pipewire initialized");
	this->thread_loop.instantiate();
	this->thread_loop->start(callable_mp(this, &PipewireLoop::poll_pw));
}

void PipewireLoop::stop() {
    // sends an interrupt to the 
    pw_loop_invoke(this->loop, do_stop, 1, NULL, 0, false, this);
}

void PipewireLoop::_thread_stop() {
    this->running = false;
}

void PipewireLoop::handle_event(Dictionary evt) {
    // TODO push events to godot signals
    this->emit_signal("pw_event", evt);
}

PipewireLoop::PipewireLoop() {
	this->loop = pw_loop_new(NULL);
}

PipewireLoop::~PipewireLoop() {
    ERR_FAIL_COND(this->loop == nullptr);

	if (this->thread_loop->is_alive()) {
        this->stop();
		this->thread_loop->wait_to_finish();
	}
	this->thread_loop->unreference();

    /** remove registry context if prepared */
    if (this->registry != nullptr) {
        pw_proxy_destroy((struct pw_proxy*)this->registry);
        pw_core_disconnect(this->core);
        pw_context_destroy(this->context);
    }

    /** remove stream if prepared */
    if (this->stream != nullptr) {
        pw_stream_destroy(this->stream);
    }
    pw_loop_destroy(this->loop);
}
