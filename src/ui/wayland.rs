use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_output, delegate_registry, delegate_seat,
    delegate_shm, delegate_layer,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Modifiers},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        wlr_layer::{
            LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    globals::GlobalList,
    protocol::{wl_keyboard, wl_output, wl_seat, wl_shm, wl_surface},
    Connection, QueueHandle,
};
use xkbcommon::xkb::{self, keysyms};
use crate::state::AppState;
use crate::ui::render::Renderer;
use crate::executor;

pub struct WaylandApp {
    pub registry_state: RegistryState,
    pub seat_state: SeatState,
    pub output_state: OutputState,
    pub compositor_state: CompositorState,
    pub shm_state: Shm,
    pub layer_shell_state: LayerShell,

    pub layer_surface: Option<LayerSurface>,
    pub pool: Option<SlotPool>,
    pub width: u32,
    pub height: u32,
    pub first_configure: bool,
    pub should_exit: bool,

    pub state: AppState,
    pub renderer: Renderer,
}

impl WaylandApp {
    pub fn new(_conn: &Connection, globals: &GlobalList, qh: &QueueHandle<Self>, state: AppState, renderer: Renderer) -> Self {
        let registry_state = RegistryState::new(globals);
        let seat_state = SeatState::new(globals, qh);
        let output_state = OutputState::new(globals, qh);
        let compositor_state = CompositorState::bind(globals, qh).expect("wl_compositor not available");
        let shm_state = Shm::bind(globals, qh).expect("wl_shm not available");
        let layer_shell_state = LayerShell::bind(globals, qh).expect("zwlr_layer_shell_v1 not available");

        Self {
            registry_state,
            seat_state,
            output_state,
            compositor_state,
            shm_state,
            layer_shell_state,
            layer_surface: None,
            pool: None,
            width: 600,
            height: 400,
            first_configure: true,
            should_exit: false,
            state,
            renderer,
        }
    }

    pub fn draw(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>) {
        if let Some(layer_surface) = &self.layer_surface {
            let width = self.width;
            let height = self.height;
            if width == 0 || height == 0 { return; }
            
            let Some(pool) = self.pool.as_mut() else { return; };

            let (buffer, canvas) = pool
                .create_buffer(
                    width as i32,
                    height as i32,
                    (width * 4) as i32,
                    wl_shm::Format::Argb8888,
                )
                .expect("create buffer");

            if let Some(mut pixmap) = tiny_skia::PixmapMut::from_bytes(canvas, width, height) {
                self.renderer.draw(&mut pixmap, &self.state);
                
                for chunk in canvas.chunks_exact_mut(4) {
                    chunk.swap(0, 2);
                }
                
                layer_surface.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
                layer_surface.wl_surface().damage(0, 0, width as i32, height as i32);
                layer_surface.wl_surface().commit();
            }
        }
    }
}

impl LayerShellHandler for WaylandApp {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.should_exit = true;
    }

    fn configure(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if configure.new_size.0 > 0 {
            self.width = configure.new_size.0;
        }
        if configure.new_size.1 > 0 {
            self.height = configure.new_size.1;
        }

        if self.first_configure {
            self.first_configure = false;
            let pool = SlotPool::new(self.width as usize * self.height as usize * 4, &self.shm_state)
                .expect("Failed to create pool");
            self.pool = Some(pool);
        }
        
        if let Some(pool) = &mut self.pool {
            if pool.len() < (self.width * self.height * 4) as usize {
                 pool.resize((self.width * self.height * 4) as usize).unwrap();
            }
        }

        self.draw(conn, qh);
    }
}

impl CompositorHandler for WaylandApp {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {}

    fn frame(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        self.draw(conn, qh);
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {}

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {}

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {}
}

impl OutputHandler for WaylandApp {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}
    fn update_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}
}

impl SeatHandler for WaylandApp {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        _capability: Capability,
    ) {
        if _capability == Capability::Keyboard && self.seat_state.get_keyboard(qh, &seat, None).is_ok() {
            // Keyboard added
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        _capability: Capability,
    ) {}

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for WaylandApp {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        _: &[xkb::Keysym],
    ) {}

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: &wl_surface::WlSurface,
        _: u32,
    ) {
        self.should_exit = true;
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _keyboard: &wl_keyboard::WlKeyboard,
        _serial: u32,
        event: KeyEvent,
    ) {
         let sym = event.keysym;
         let raw_sym = u32::from(sym);
         
         match raw_sym {
            keysyms::KEY_Escape => self.should_exit = true,
            keysyms::KEY_Return => {
                 if let Some(entry) = self.state.get_selected() {
                     let _ = executor::execute(entry, &self.state.config, &self.state.active_group);
                     self.should_exit = true;
                 }
            }
            keysyms::KEY_Up => self.state.move_selection(-1),
            keysyms::KEY_Down => self.state.move_selection(1),
            keysyms::KEY_BackSpace => {
                self.state.query.pop();
                self.state.update_query(&self.state.query.clone());
            }
            keysyms::KEY_1 | keysyms::KEY_2 | keysyms::KEY_3 |
            keysyms::KEY_4 | keysyms::KEY_5 | keysyms::KEY_6 |
            keysyms::KEY_7 | keysyms::KEY_8 | keysyms::KEY_9 => {
                let index_offset = (raw_sym - keysyms::KEY_1) as usize;
                
                let item_height = 30.0;
                let list_start_y = self.state.config.theme.padding + 20.0 + self.state.config.theme.spacing;
                let visible_items = (self.height as f32 - list_start_y - self.state.config.theme.padding) / item_height;
                let visible_items = visible_items as usize;
                
                let total_items = self.state.filtered_indices.len();
                let scroll_offset = if total_items <= visible_items {
                    0
                } else {
                     if self.state.selected_index < visible_items / 2 {
                         0
                     } else if self.state.selected_index >= total_items - visible_items / 2 {
                         total_items.saturating_sub(visible_items)
                     } else {
                         self.state.selected_index - visible_items / 2
                     }
                };

                let target_index = scroll_offset + index_offset;
                if let Some(&entry_idx) = self.state.filtered_indices.get(target_index) {
                    let entry = &self.state.entries[entry_idx];
                    let _ = executor::execute(entry, &self.state.config, &self.state.active_group);
                    self.should_exit = true;
                }
            }
            _ => {
                if let Some(utf8) = event.utf8 {
                     if !utf8.chars().any(|c| c.is_control()) {
                         self.state.query.push_str(&utf8);
                         self.state.update_query(&self.state.query.clone());
                     }
                }
            }
         }
         
         if let Some(layer_surface) = &self.layer_surface {
             layer_surface.wl_surface().frame(qh, layer_surface.wl_surface().clone());
             layer_surface.wl_surface().commit();
         }
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        _: KeyEvent,
    ) {}

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _serial: u32,
        _modifiers: Modifiers,
        _layout: u32,
    ) {}
}


impl ShmHandler for WaylandApp {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm_state
    }
}

delegate_compositor!(WaylandApp);
delegate_output!(WaylandApp);
delegate_shm!(WaylandApp);
delegate_seat!(WaylandApp);
delegate_keyboard!(WaylandApp);
delegate_layer!(WaylandApp);
delegate_registry!(WaylandApp);

impl ProvidesRegistryState for WaylandApp {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    
    fn runtime_add_global(&mut self, _: &Connection, _: &QueueHandle<Self>, _: u32, _: &str, _: u32) {
    }
    fn runtime_remove_global(&mut self, _: &Connection, _: &QueueHandle<Self>, _: u32, _: &str) {
    }
}
