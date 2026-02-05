use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tiny_skia::{Pixmap, Transform};
use image::ImageReader;
use std::fs;
use std::sync::mpsc::{Sender, channel};
use std::thread;

pub struct IconCache {
    pub cache: HashMap<String, Option<Pixmap>>,
    pending: HashSet<String>,
    icon_theme_paths: Vec<PathBuf>,
    request_tx: Sender<(String, u32)>,
}

impl IconCache {
    pub fn new(response_tx: calloop::channel::Sender<(String, Option<Pixmap>)>) -> Self {
        let mut paths = Vec::new();
        if let Some(home) = directories::BaseDirs::new() {
            paths.push(home.data_dir().join("icons"));
        }
        paths.push(PathBuf::from("/usr/share/icons"));
        paths.push(PathBuf::from("/usr/share/pixmaps"));
        
        let (request_tx, request_rx) = channel::<(String, u32)>();

        let paths_clone = paths.clone();
        thread::spawn(move || {
            let loader = IconLoader { icon_theme_paths: paths_clone };
            while let Ok((icon_name, size)) = request_rx.recv() {
                let pixmap = loader.find_and_load(&icon_name, size);
                let _ = response_tx.send((icon_name, pixmap));
            }
        });

        Self {
            cache: HashMap::new(),
            pending: HashSet::new(),
            icon_theme_paths: paths,
            request_tx,
        }
    }

    pub fn get(&mut self, icon_name: &str, size: u32) -> Option<Pixmap> {
        if let Some(cached) = self.cache.get(icon_name) {
            return cached.clone();
        }

        if !self.pending.contains(icon_name) {
            self.pending.insert(icon_name.to_string());
            let _ = self.request_tx.send((icon_name.to_string(), size));
        }

        None
    }

    pub fn insert(&mut self, name: String, pixmap: Option<Pixmap>) {
        self.cache.insert(name.clone(), pixmap);
        self.pending.remove(&name);
    }
}

struct IconLoader {
    icon_theme_paths: Vec<PathBuf>,
}

impl IconLoader {
    fn find_and_load(&self, icon_name: &str, size: u32) -> Option<Pixmap> {
        let path = Path::new(icon_name);
        if path.is_absolute() && path.exists() {
             return self.load_from_path(path, size);
        }

        for root in &self.icon_theme_paths {
            if !root.exists() { continue; }
            
            let common_subdirs = [
                "hicolor/48x48/apps",
                "hicolor/scalable/apps",
                "hicolor/32x32/apps",
                "hicolor/64x64/apps",
                "Adwaita/48x48/apps",
                "Adwaita/scalable/apps",
                "",
            ];

            for sub in common_subdirs {
                let dir = root.join(sub);
                if !dir.exists() { continue; }
                
                let extensions = ["png", "svg", "xpm"];
                for ext in extensions {
                    let file_path = dir.join(format!("{}.{}", icon_name, ext));
                    if file_path.exists() {
                        return self.load_from_path(&file_path, size);
                    }
                }
            }
        }
        None
    }

    fn load_from_path(&self, path: &Path, size: u32) -> Option<Pixmap> {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        match ext {
            "svg" => self.load_svg(path, size),
            _ => self.load_raster(path, size),
        }
    }

    fn load_raster(&self, path: &Path, size: u32) -> Option<Pixmap> {
        let img = ImageReader::open(path).ok()?.decode().ok()?;
        let img = img.resize(size, size, image::imageops::FilterType::Lanczos3);
        let mut rgba = img.into_rgba8();
        
        for pixel in rgba.chunks_exact_mut(4) {
            let a = pixel[3] as f32 / 255.0;
            pixel[0] = (pixel[0] as f32 * a) as u8;
            pixel[1] = (pixel[1] as f32 * a) as u8;
            pixel[2] = (pixel[2] as f32 * a) as u8;
        }

        let width = rgba.width();
        let height = rgba.height();
        
        Pixmap::from_vec(rgba.into_vec(), tiny_skia::IntSize::from_wh(width, height)?)
    }

    fn load_svg(&self, path: &Path, size: u32) -> Option<Pixmap> {
        let opt = resvg::usvg::Options::default();
        let svg_data = fs::read(path).ok()?;
        let tree = resvg::usvg::Tree::from_data(&svg_data, &opt).ok()?;

        let mut pixmap = Pixmap::new(size, size)?;
        let transform = Transform::from_scale(
            size as f32 / tree.size().width(),
            size as f32 / tree.size().height(),
        );
        
        resvg::render(&tree, transform, &mut pixmap.as_mut());
        Some(pixmap)
    }
}