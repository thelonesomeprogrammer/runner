use tiny_skia::{Paint, Color, Rect, Transform, PixmapMut, PixmapPaint, PathBuilder, Stroke};
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, SwashCache};
use crate::state::AppState;
use crate::ui::icons::IconCache;
use crate::config::ThemeConfig;

pub struct Renderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    pub icon_cache: IconCache,
}

impl Renderer {
    pub fn new(icon_cache: IconCache) -> Self {
        Self {
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
            icon_cache,
        }
    }

    pub fn insert_icon(&mut self, name: String, pixmap: Option<tiny_skia::Pixmap>) {
        self.icon_cache.insert(name, pixmap);
    }

    pub fn draw(&mut self, pixmap: &mut PixmapMut, state: &AppState) {
        let theme = &state.config.theme;
        let bg_color = ThemeConfig::parse_color(&theme.background);
        let border_color = ThemeConfig::parse_color(&theme.border_color);
        let text_color = ThemeConfig::parse_color(&theme.text);
        let sel_bg_color = ThemeConfig::parse_color(&theme.selection_background);
        let sel_text_color = ThemeConfig::parse_color(&theme.selection_text);

        pixmap.fill(Color::TRANSPARENT);

        let width = pixmap.width() as f32;
        let height = pixmap.height() as f32;

        let rect = Rect::from_xywh(0.0, 0.0, width, height).unwrap();
        self.draw_rounded_rect(pixmap, rect, theme.border_radius, bg_color, Some(border_color));

        let search_y = theme.padding;
        let search_text = if state.query.is_empty() {
            "Search apps...".to_string()
        } else {
            format!("> {}", state.query)
        };
        let search_color = if state.query.is_empty() {
            Color::from_rgba8(100, 100, 100, 255)
        } else {
            text_color
        };

        self.draw_text(pixmap, &search_text, theme.padding, search_y, 20.0, search_color);

        let item_height = 30.0; 
        let list_start_y = search_y + 20.0 + theme.spacing;
        
        let visible_items = (height - list_start_y - theme.padding) / item_height;
        let visible_items = visible_items as usize;
        
        let total_items = state.filtered_indices.len();
        let scroll_offset = if total_items <= visible_items {
            0
        } else {
             if state.selected_index < visible_items / 2 {
                 0
             } else if state.selected_index >= total_items - visible_items / 2 {
                 total_items.saturating_sub(visible_items)
             } else {
                 state.selected_index - visible_items / 2
             }
        };

        for (i, &entry_idx) in state.filtered_indices.iter().enumerate().skip(scroll_offset).take(visible_items) {
            let entry = &state.entries[entry_idx];
            let relative_index = i - scroll_offset;
            let y = list_start_y + (relative_index as f32 * item_height);
            
            let mut current_text_color = text_color;

            if i == state.selected_index {
                let sel_rect = Rect::from_xywh(theme.padding / 2.0, y, width - theme.padding, item_height).unwrap();
                self.draw_rounded_rect(pixmap, sel_rect, theme.border_radius / 2.0, sel_bg_color, None);
                current_text_color = sel_text_color;
            }
            
            let mut text_x = theme.padding;
            if relative_index < 9 {
                let nr_text = format!("{}. ", relative_index + 1);
                let num_color = ThemeConfig::parse_color(&theme.number_color);
                self.draw_text(pixmap, &nr_text, theme.padding, y + (item_height - 16.0) / 2.0, 14.0, num_color);
                text_x += 20.0;
            }

            let icon_size = 22;
            let icon_padding = 10.0;
            
            if let Some(icon_name) = &entry.icon {
                if let Some(icon_pixmap) = self.icon_cache.get(icon_name, icon_size) {
                    let icon_paint = PixmapPaint::default();
                    pixmap.draw_pixmap(text_x as i32, (y + (item_height - icon_size as f32) / 2.0) as i32, icon_pixmap.as_ref(), &icon_paint, Transform::identity(), None);
                    text_x += icon_size as f32 + icon_padding;
                }
            }

            self.draw_text(pixmap, &entry.name, text_x, y + (item_height - 16.0) / 2.0, 16.0, current_text_color);
        }

        if state.filtered_indices.is_empty() {
             self.draw_text(pixmap, "No results found", theme.padding, list_start_y, 16.0, Color::from_rgba8(150, 100, 100, 255));
        }
    }

    fn draw_rounded_rect(&self, pixmap: &mut PixmapMut, rect: Rect, radius: f32, fill: Color, stroke: Option<Color>) {
        let mut pb = PathBuilder::new();
        let x = rect.left();
        let y = rect.top();
        let w = rect.width();
        let h = rect.height();

        pb.move_to(x + radius, y);
        pb.line_to(x + w - radius, y);
        pb.quad_to(x + w, y, x + w, y + radius);
        pb.line_to(x + w, y + h - radius);
        pb.quad_to(x + w, y + h, x + w - radius, y + h);
        pb.line_to(x + radius, y + h);
        pb.quad_to(x, y + h, x, y + h - radius);
        pb.line_to(x, y + radius);
        pb.quad_to(x, y, x + radius, y);
        pb.close();

        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(fill);
            paint.anti_alias = true;
            pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);

            if let Some(s_color) = stroke {
                let mut s_paint = Paint::default();
                s_paint.set_color(s_color);
                s_paint.anti_alias = true;
                let stroke_obj = Stroke { width: 1.5, ..Default::default() };
                pixmap.stroke_path(&path, &s_paint, &stroke_obj, Transform::identity(), None);
            }
        }
    }

    fn draw_text(&mut self, pixmap: &mut PixmapMut, text: &str, x: f32, y: f32, size: f32, color: Color) {
        let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(size, size));
        buffer.set_size(&mut self.font_system, Some(pixmap.width() as f32 - x), None);
        buffer.set_text(&mut self.font_system, text, Attrs::new(), cosmic_text::Shaping::Advanced);
        buffer.shape_until_scroll(&mut self.font_system, false);

        let text_color = cosmic_text::Color::rgba(
            (color.red() * 255.0) as u8,
            (color.green() * 255.0) as u8,
            (color.blue() * 255.0) as u8,
            (color.alpha() * 255.0) as u8,
        );

        buffer.draw(&mut self.font_system, &mut self.swash_cache, text_color, |draw_x, draw_y, w, h, color| {
            let draw_x = draw_x + x as i32;
            let draw_y = draw_y + y as i32;
            if w == 0 || h == 0 { return; }
            if draw_x >= 0 && draw_y >= 0 && draw_x < pixmap.width() as i32 && draw_y < pixmap.height() as i32 {
                 let paint = Paint {
                    shader: tiny_skia::Shader::SolidColor(tiny_skia::Color::from_rgba8(color.r(), color.g(), color.b(), color.a())),
                    ..Paint::default()
                };
                let rect = Rect::from_xywh(draw_x as f32, draw_y as f32, w as f32, h as f32);
                if let Some(r) = rect {
                    pixmap.fill_rect(r, &paint, Transform::identity(), None);
                }
            }
        });
    }
}