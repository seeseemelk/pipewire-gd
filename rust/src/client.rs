use std::collections::HashSet;
use godot::builtin::{Array, GString};
use godot::prelude::*;

use libspa::ForeignDict;
use pipewire as pw;

use crate::stream::PipewireStream;

struct Terminate;

#[derive(GodotClass)]
#[class(tool, base=Node)]
pub struct PipewireClient {
    pub pw_loop: pw::Loop,
    pub pw_core: pw::Core,

    sources_receiver: std::sync::mpsc::Receiver<u32>,
    available_sources: HashSet<u32>,

    #[base]
    base: Base<Node>,
}

#[godot_api]
impl INode for PipewireClient {
    fn init(base: Base<Node>) -> Self {
        // starts a pipewire main loop
        // we will manage this in a parallel thread to godot's main thread
        let Ok(loop_) = pw::Loop::new() else { panic!(); };
        let Ok(context) = pw::Context::new(&loop_) else { panic!(); };
        let Ok(core) = context.connect(None) else { panic!(); };
        let Ok(_registry) = core.get_registry() else { panic!(); };
        let (_sender, receiver) = std::sync::mpsc::channel();

        let available_sources = HashSet::new();

        // TODO listen for list of sources
        // let _sourcesListener = registry
        //    .add_listener_local()
        //    .global(Self::handle_source_change)
        //    .register();


        Self {
            pw_loop: loop_,
            pw_core: core,
            sources_receiver: receiver,
            available_sources: available_sources,
            base,
        }
    }
}

const SIGNAL_SOURCE_CHANGED: &str = "source_changed";

impl Drop for PipewireClient {
    fn drop(&mut self) {
        //drop(self.sources_listener);
    }
}

#[godot_api]
impl PipewireClient {
    fn process(&self, _delta: f64) {
        // loop over pipewire sources each process frame
        // limit to 5ms so as not to eat up too much frame time
        self.pw_loop.iterate(std::time::Duration::from_millis(5));
    }

    #[func]
    fn enumerate(&mut self) -> Array<GString> {
        let mut arr = Array::new();
        
        for global in &self.available_sources {
            arr.push(GString::from(format!("{}", global)));
        }
        
        return arr;
    }

    #[func]
    fn connect_source(&mut self, source_id: u32) {
        let name = GString::from(format!("{:?}", source_id));
        let path = NodePath::from(name);
        if let Some(_) = self.base().get_node(path) { 
            // do not create a new stream node if one already exists
            return; 
        }

        let mut node = PipewireStream::new_alloc();
        node.bind_mut().source_id = source_id;

        self.base_mut().add_child(node.upcast());
    }

    fn handle_source_change(&mut self, global: pw::registry::GlobalObject<ForeignDict>) {
        println!("source detected {:?}", global);

        self.available_sources.insert(global.id);
        let name = format!("{}", global.id);
        self.base_mut().emit_signal(
            StringName::from(SIGNAL_SOURCE_CHANGED),
            &[GString::from(name).to_variant()]
        );
    }
}
