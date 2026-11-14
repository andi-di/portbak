use super::{noise, sprite_point, texture, Garden, Sprite, SpritePose};
use eframe::egui::{self, Color32, Rect};
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

pub(super) struct Item {
    bounds: Rect,
    shape: egui::Shape,
}

pub(super) struct Frame {
    pass: u64,
    pub garden: Garden,
    pub scene: OnceLock<Vec<Item>>,
    pub bats: OnceLock<Vec<Item>>,
    pub petals: OnceLock<Vec<Item>>,
    pub particles: OnceLock<Vec<Item>>,
}

impl Frame {
    pub fn new(pass: u64, garden: Garden) -> Self {
        Self {
            pass,
            garden,
            scene: OnceLock::new(),
            bats: OnceLock::new(),
            petals: OnceLock::new(),
            particles: OnceLock::new(),
        }
    }
}

pub(super) fn frame_id() -> egui::Id {
    egui::Id::new("neko_prepared_frame")
}

pub(super) fn frame(ctx: &egui::Context) -> Arc<Frame> {
    let pass = ctx.cumulative_pass_nr();
    if let Some(frame) = ctx.data_mut(|data| data.get_temp::<Arc<Frame>>(frame_id())) {
        if frame.pass == pass {
            return frame;
        }
    }
    let garden = ctx
        .data_mut(|data| data.get_temp::<Garden>(super::state_id()))
        .unwrap_or_default();
    let frame = Arc::new(Frame::new(pass, garden));
    ctx.data_mut(|data| data.insert_temp(frame_id(), frame.clone()));
    frame
}

pub(super) fn paint_items(painter: &egui::Painter, items: &[Item]) {
    let clip = painter.clip_rect();
    painter.extend(
        items
            .iter()
            .filter(|item| item.bounds.intersects(clip))
            .map(|item| item.shape.clone()),
    );
}

pub(super) struct SceneBuilder<'a> {
    pub ctx: &'a egui::Context,
    clip: Rect,
    textures: HashMap<Sprite, Option<egui::TextureHandle>>,
    items: Vec<Item>,
}

impl<'a> SceneBuilder<'a> {
    pub fn new(ctx: &'a egui::Context, clip: Rect) -> Self {
        Self {
            ctx,
            clip,
            textures: HashMap::new(),
            items: Vec::new(),
        }
    }

    pub fn finish(self) -> Vec<Item> {
        self.items
    }

    pub fn visible(&self, bounds: Rect) -> bool {
        self.clip.intersects(bounds)
    }

    pub fn texture(&mut self, sprite: Sprite) -> Option<egui::TextureHandle> {
        self.textures
            .entry(sprite)
            .or_insert_with(|| texture(self.ctx, sprite))
            .clone()
    }

    pub fn mesh(&mut self, bounds: Rect, mesh: egui::Mesh) {
        self.items.push(Item {
            bounds,
            shape: egui::Shape::mesh(mesh),
        });
    }

    pub fn rect(&mut self, bounds: Rect, radius: impl Into<egui::CornerRadius>, color: Color32) {
        if self.visible(bounds) {
            self.items.push(Item {
                bounds,
                shape: egui::Shape::rect_filled(bounds, radius, color),
            });
        }
    }

    pub fn draw(&mut self, rect: Rect, sprite: Sprite, tint: Color32, pose: SpritePose) {
        let margin = rect.height() * pose.flap.abs() * 0.18
            + rect.width().max(rect.height()) * pose.angle.sin().abs();
        if !self.visible(rect.expand(margin)) {
            return;
        }
        let Some(texture) = self.texture(sprite) else {
            return;
        };
        let mut mesh = egui::Mesh::with_texture(texture.id());
        for v in [0.0, 1.0] {
            for u in [0.0, 0.4, 0.6, 1.0] {
                mesh.vertices.push(egui::epaint::Vertex {
                    pos: sprite_point(rect, u, v, &pose),
                    uv: egui::pos2(if pose.mirrored { 1.0 - u } else { u }, v),
                    color: tint,
                });
            }
        }
        for column in 0..3 {
            mesh.add_triangle(column, column + 1, column + 4);
            mesh.add_triangle(column + 1, column + 5, column + 4);
        }
        self.mesh(mesh.calc_bounds(), mesh);
    }

    pub fn castle_windows(&mut self, rect: Rect, opacity: f32) {
        if !self.visible(rect) {
            return;
        }
        let Some(texture) = self.texture(Sprite::CastleWindows) else {
            return;
        };
        static CELLS: OnceLock<Vec<[u32; 2]>> = OnceLock::new();
        let cells = CELLS.get_or_init(|| {
            serde_json::from_slice(include_bytes!(
                "../../../../assets/themes/midnight-neko/scenery/castle-windows.json"
            ))
            .expect("embedded castle window cells")
        });
        let phase = self.ctx.input(|input| (input.time * 0.7) as u32 % 8);
        let mut mesh = egui::Mesh::with_texture(texture.id());
        let size = texture.size_vec2();
        for [x, y] in cells {
            if noise(*x + *y * 100, phase) <= 0.85 {
                continue;
            }
            let uv = Rect::from_min_size(
                egui::pos2(*x as f32 * 16.0 / size.x, *y as f32 * 16.0 / size.y),
                egui::vec2(16.0 / size.x, 16.0 / size.y),
            );
            let target = Rect::from_min_max(
                rect.min + rect.size() * uv.min.to_vec2(),
                rect.min + rect.size() * uv.max.to_vec2(),
            );
            if self.visible(target) {
                mesh.add_rect_with_uv(target, uv, Color32::WHITE.gamma_multiply(opacity));
            }
        }
        if !mesh.is_empty() {
            self.mesh(rect, mesh);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipped_sprites_do_not_load_textures_or_build_meshes() {
        let ctx = egui::Context::default();
        let mut scene = SceneBuilder::new(
            &ctx,
            Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(100.0)),
        );
        scene.draw(
            Rect::from_min_size(egui::pos2(500.0, 500.0), egui::Vec2::splat(40.0)),
            Sprite::Castle,
            Color32::WHITE,
            SpritePose::default(),
        );
        assert!(scene.textures.is_empty());
        assert!(scene.finish().is_empty());
    }

    #[test]
    fn prepared_meshes_are_shared_between_clipped_panels() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            super::super::paint_background(&ctx, crate::app::theme::style::Palette::of_ctx(&ctx));
            let a = frame(&ctx);
            let b = frame(&ctx);
            assert!(Arc::ptr_eq(&a, &b));
            super::super::paint_bats(ui.painter());
            let items = a.bats.get().unwrap();
            assert!(!items.is_empty());
            let address = items.as_ptr();
            super::super::paint_bats(&ui.painter().with_clip_rect(ui.clip_rect().shrink(10.0)));
            assert_eq!(address, a.bats.get().unwrap().as_ptr());
        });
    }

    #[test]
    fn scenery_textures_are_shared_across_brightness_and_flicker_phases() {
        let ctx = egui::Context::default();
        let mut ids = Vec::new();
        for phase in 0..8 {
            ctx.set_theme(if phase % 2 == 0 {
                egui::Theme::Dark
            } else {
                egui::Theme::Light
            });
            let _ = ctx.run_ui(
                egui::RawInput {
                    time: Some(phase as f64 * 2.0),
                    ..Default::default()
                },
                |_| {
                    ids.push((
                        texture(&ctx, Sprite::Castle).unwrap().id(),
                        texture(&ctx, Sprite::CastleWindows).unwrap().id(),
                    ));
                },
            );
        }
        assert!(ids.iter().all(|id| *id == ids[0]));
    }

    #[test]
    fn minimized_and_occluded_windows_skip_decorative_work() {
        for minimized in [true, false] {
            let ctx = egui::Context::default();
            let mut input = egui::RawInput::default();
            let viewport = input.viewports.get_mut(&egui::ViewportId::ROOT).unwrap();
            if minimized {
                viewport.minimized = Some(true);
            } else {
                viewport.occluded = Some(true);
            }
            let _ = ctx.run_ui(input, |ui| {
                assert!(!super::super::decorations_visible(&ctx));
                super::super::paint_background(
                    &ctx,
                    crate::app::theme::style::Palette::of_ctx(&ctx),
                );
                super::super::paint_bats(ui.painter());
                assert!(ctx
                    .data_mut(|data| data.get_temp::<Arc<Frame>>(frame_id()))
                    .is_none());
                assert!(ctx
                    .data_mut(|data| data.get_temp::<Garden>(super::super::state_id()))
                    .is_none());
            });
        }
    }
}
