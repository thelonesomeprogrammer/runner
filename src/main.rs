mod config;
mod state;
mod model;
mod sources;
mod matcher;
mod ui;
mod executor;

use anyhow::Result;
use calloop::EventLoop;
use calloop_wayland_source::WaylandSource;
use smithay_client_toolkit::{
    shell::wlr_layer::{Layer, KeyboardInteractivity, Anchor},
    shell::WaylandSurface,
};
use wayland_client::{Connection, globals::registry_queue_init};
use crate::config::load_config;
use crate::state::AppState;
use crate::ui::wayland::WaylandApp;
use crate::ui::render::Renderer;
use crate::ui::icons::IconCache;
use crate::sources::{Source, desktop::DesktopSource, bin::BinSource, scripts::ScriptsSource};
use crate::model::{Entry, EntryType};
use std::thread;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Launch group to use
    #[arg(short, long, default_value = "default")]
    group: String,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    // 1. Load Config
    let config = load_config()?;
    
    // Validate group exists, fallback to default if not
    let group_name = if config.groups.contains_key(&args.group) {
        args.group.clone()
    } else {
        "default".to_string()
    };
    
    let group_config = config.groups.get(&group_name).cloned().unwrap_or_default();

    // 2. Setup Wayland Connection & Event Loop
    let mut event_loop: EventLoop<WaylandApp> = EventLoop::try_new()?;
    let conn = Connection::connect_to_env()?;
    let (globals, event_queue) = registry_queue_init::<WaylandApp>(&conn).unwrap();
    let qh = event_queue.handle();

    // 3. Init State & UI
    let (tx_icons, rx_icons) = calloop::channel::channel::<(String, Option<tiny_skia::Pixmap>)>();
    let icon_cache = IconCache::new(tx_icons);
    let renderer = Renderer::new(icon_cache);

    let mut app_state = AppState::new(config.clone());
    app_state.active_group = group_name; 
    let mut app = WaylandApp::new(&conn, &globals, &qh, app_state, renderer);

    // 4. Create Layer Surface
    let surface = app.compositor_state.create_surface(&qh);
    let layer_surface = app.layer_shell_state.create_layer_surface(
        &qh,
        surface,
        Layer::Overlay,
        Some("runner"),
        None,
    );
    
    layer_surface.set_anchor(Anchor::empty()); 
    layer_surface.set_size(config.theme.width, config.theme.height);
    layer_surface.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
    layer_surface.commit(); 
    app.layer_surface = Some(layer_surface);

    // 5. Spawn Source Loader based on Group config
    let (tx_entries, rx_entries) = calloop::channel::channel();
    let sources_to_scan = group_config.sources.clone();
    let static_items = group_config.items.clone();
    
    thread::spawn(move || {
        let mut entries = Vec::new();
        
        // Add static items
        for item in static_items {
            let mut entry = Entry::new(
                format!("custom:{}", item.name),
                item.name,
                item.command,
                EntryType::Custom,
                item.terminal,
            );
            entry.icon = item.icon;
            entries.push(entry);
        }

        // Only scan if the source is in the group's source list
        if sources_to_scan.contains(&"desktop".to_string()) {
            if let Ok(mut e) = DesktopSource.scan() {
                 entries.append(&mut e);
            }
        }
        if sources_to_scan.contains(&"bin".to_string()) {
             if let Ok(mut e) = BinSource.scan() {
                 entries.append(&mut e);
             }
        }
        if sources_to_scan.contains(&"scripts".to_string()) {
             if let Ok(mut e) = ScriptsSource.scan() {
                 entries.append(&mut e);
             }
        }
        let _ = tx_entries.send(entries);
    });

    let conn_clone = conn.clone();
    let qh_clone = qh.clone();

    // Icon update handler
    let conn_c1 = conn_clone.clone();
    let qh_c1 = qh_clone.clone();
    event_loop.handle().insert_source(rx_icons, move |event, _, app: &mut WaylandApp| {
        if let calloop::channel::Event::Msg((name, pixmap)) = event {
            app.renderer.insert_icon(name, pixmap);
            app.draw(&conn_c1, &qh_c1);
        }
    }).unwrap();

    // Entry loader handler
    let conn_c2 = conn_clone.clone();
    let qh_c2 = qh_clone.clone();
    event_loop.handle().insert_source(rx_entries, move |event, _, app: &mut WaylandApp| {
        if let calloop::channel::Event::Msg(entries) = event {
            app.state.set_entries(entries);
            app.draw(&conn_c2, &qh_c2);
        }
    }).unwrap();
    
    event_loop.handle().insert_source(
        WaylandSource::new(conn.clone(), event_queue),
        |_, queue, app| {
            queue.dispatch_pending(app)
        }
    ).unwrap();

    // 6. Run Loop
    loop {
        if app.should_exit {
            break;
        }
        event_loop.dispatch(None, &mut app)?;
    }

    Ok(())
}