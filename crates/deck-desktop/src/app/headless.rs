//! Render the real egui widget tree without a display server. No generated concept art.
use super::*;
use std::collections::HashMap;
fn draw(
    ctx: &Context,
    app: &mut Studio,
    size: Vec2,
    events: Vec<eframe::egui::Event>,
) -> FullOutput {
    ctx.run(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            events,
            ..Default::default()
        },
        |ctx| app.render(ctx),
    )
}
fn ready() -> (Context, Studio, tempfile::TempDir) {
    let ctx = Context::default();
    let dir = tempfile::tempdir().unwrap();
    let mut app = Studio::create(&ctx, true, Some(dir.path().into()));
    for _ in 0..100 {
        app.events();
        if app.ready {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(app.ready, "runtime startup failed");
    (ctx, app, dir)
}
fn texture_updates(textures: &mut HashMap<TextureId, ColorImage>, out: &FullOutput) {
    for (id, delta) in &out.textures_delta.set {
        let eframe::egui::ImageData::Color(im) = &delta.image;
        if let Some([x, y]) = delta.pos {
            let image = textures.get_mut(id).unwrap();
            for row in 0..im.size[1] {
                let dst = (row + y) * image.size[0] + x;
                let src = row * im.size[0];
                image.pixels[dst..dst + im.size[0]]
                    .copy_from_slice(&im.pixels[src..src + im.size[0]]);
            }
        } else {
            textures.insert(*id, (**im).clone());
        }
    }
}
fn raster(
    ctx: &Context,
    out: FullOutput,
    textures: &HashMap<TextureId, ColorImage>,
    size: Vec2,
    path: &std::path::Path,
) {
    let width = size.x as u32;
    let height = size.y as u32;
    let mut canvas = image::RgbaImage::from_pixel(width, height, image::Rgba([17, 20, 25, 255]));
    let meshes = ctx.tessellate(out.shapes, 1.0);
    for c in meshes {
        let eframe::egui::epaint::Primitive::Mesh(mesh) = c.primitive else {
            continue;
        };
        let Some(tex) = textures.get(&mesh.texture_id) else {
            continue;
        };
        for tri in mesh.indices.as_chunks::<3>().0 {
            let a = mesh.vertices[tri[0] as usize];
            let b = mesh.vertices[tri[1] as usize];
            let c0 = mesh.vertices[tri[2] as usize];
            let minx = a
                .pos
                .x
                .min(b.pos.x)
                .min(c0.pos.x)
                .floor()
                .max(c.clip_rect.min.x)
                .max(0.0) as u32;
            let maxx = a
                .pos
                .x
                .max(b.pos.x)
                .max(c0.pos.x)
                .ceil()
                .min(c.clip_rect.max.x)
                .min(width as f32) as u32;
            let miny = a
                .pos
                .y
                .min(b.pos.y)
                .min(c0.pos.y)
                .floor()
                .max(c.clip_rect.min.y)
                .max(0.0) as u32;
            let maxy = a
                .pos
                .y
                .max(b.pos.y)
                .max(c0.pos.y)
                .ceil()
                .min(c.clip_rect.max.y)
                .min(height as f32) as u32;
            let area = (b.pos.x - a.pos.x) * (c0.pos.y - a.pos.y)
                - (b.pos.y - a.pos.y) * (c0.pos.x - a.pos.x);
            if area.abs() < 0.0001 {
                continue;
            }
            for y in miny..maxy {
                for x in minx..maxx {
                    let p = pos2(x as f32 + 0.5, y as f32 + 0.5);
                    let wa = ((b.pos.x - p.x) * (c0.pos.y - p.y)
                        - (b.pos.y - p.y) * (c0.pos.x - p.x))
                        / area;
                    let wb = ((c0.pos.x - p.x) * (a.pos.y - p.y)
                        - (c0.pos.y - p.y) * (a.pos.x - p.x))
                        / area;
                    let wc = 1.0 - wa - wb;
                    if wa < 0.0 || wb < 0.0 || wc < 0.0 {
                        continue;
                    }
                    let uv = a.uv.to_vec2() * wa + b.uv.to_vec2() * wb + c0.uv.to_vec2() * wc;
                    let u = (uv.x * tex.size[0] as f32 - 0.5).clamp(0.0, tex.size[0] as f32 - 1.0);
                    let v = (uv.y * tex.size[1] as f32 - 0.5).clamp(0.0, tex.size[1] as f32 - 1.0);
                    let tx = u.floor() as usize;
                    let ty = v.floor() as usize;
                    let fx = u - tx as f32;
                    let fy = v - ty as f32;
                    let mut tc = [0.0; 4];
                    for (dy, wy) in [(0, 1.0 - fy), (1, fy)] {
                        for (dx, wx) in [(0, 1.0 - fx), (1, fx)] {
                            let col = tex.pixels[(ty + dy).min(tex.size[1] - 1) * tex.size[0]
                                + (tx + dx).min(tex.size[0] - 1)]
                            .to_array();
                            for k in 0..4 {
                                tc[k] += col[k] as f32 * wx * wy;
                            }
                        }
                    }
                    let ca = a.color.to_array();
                    let cb = b.color.to_array();
                    let cc = c0.color.to_array();
                    let mut src = [0.0; 4];
                    for k in 0..4 {
                        src[k] = (ca[k] as f32 * wa + cb[k] as f32 * wb + cc[k] as f32 * wc)
                            * tc[k]
                            / 255.0;
                    }
                    let dst = canvas.get_pixel_mut(x, y);
                    for k in 0..3 {
                        dst[k] = (src[k] + dst[k] as f32 * (1.0 - src[3] / 255.0)).clamp(0.0, 255.0)
                            as u8;
                    }
                }
            }
        }
    }
    canvas.save(path).unwrap();
}
#[test]
fn native_ui_renders_all_pages_and_key_selection() {
    let (ctx, mut app, _dir) = ready();
    let size = vec2(1360.0, 900.0);
    let mut textures = HashMap::new();
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/verification");
    std::fs::create_dir_all(&folder).unwrap();
    for (page, name) in [
        (Page::Deck, "native-deck"),
        (Page::Display, "native-oled"),
        (Page::Home, "native-home-assistant"),
        (Page::Activity, "native-activity"),
        (Page::Settings, "native-settings"),
    ] {
        app.page = page;
        for _ in 0..2 {
            let o = draw(&ctx, &mut app, size, vec![]);
            texture_updates(&mut textures, &o);
        }
        let o = draw(&ctx, &mut app, size, vec![]);
        texture_updates(&mut textures, &o);
        assert!(!o.shapes.is_empty());
        raster(
            &ctx,
            o,
            &textures,
            size,
            &folder.join(format!("{name}.png")),
        );
    }
    app.page = Page::Deck;
    let o = draw(&ctx, &mut app, size, vec![]);
    let position = o
        .shapes
        .iter()
        .find_map(|s| {
            if let eframe::egui::epaint::Shape::Text(t) = &s.shape {
                if t.galley.text() == "K7" {
                    Some(t.pos + vec2(2.0, 2.0))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .expect("K7 visible in actual widget tree");
    let events = vec![
        eframe::egui::Event::PointerMoved(position),
        eframe::egui::Event::PointerButton {
            pos: position,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        },
        eframe::egui::Event::PointerButton {
            pos: position,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        },
    ];
    let _ = draw(&ctx, &mut app, size, events);
    assert_eq!(app.key, 7, "native button interaction selects K7");
    // Compact layouts use scroll panels; this is geometry QA, not OS/window-manager testing.
    for _ in 0..2 {
        let o = draw(&ctx, &mut app, vec2(1050.0, 720.0), vec![]);
        texture_updates(&mut textures, &o);
    }
    let o = draw(&ctx, &mut app, vec2(1050.0, 720.0), vec![]);
    texture_updates(&mut textures, &o);
    raster(
        &ctx,
        o,
        &textures,
        vec2(1050.0, 720.0),
        &folder.join("native-compact.png"),
    );
}
