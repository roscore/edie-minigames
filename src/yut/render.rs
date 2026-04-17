//! Board + HUD rendering for EDIE Yut Nori (macroquad).

use crate::assets::AssetHandles;
use crate::render::camera::Camera;
use crate::yut::board::*;
use crate::yut::game::*;
use macroquad::prelude::*;

const YUT_W: f32 = 1280.0;
const YUT_H: f32 = 720.0;

const ORANGE: Color = Color::new(0.98, 0.66, 0.32, 1.0);
const GREEN: Color = Color::new(0.46, 0.95, 0.74, 1.0);
const PURPLE: Color = Color::new(0.72, 0.56, 0.92, 1.0);
const CYAN: Color = Color::new(0.42, 0.84, 0.99, 1.0);
const WARM_TEXT: Color = Color::new(0.28, 0.18, 0.10, 1.0);
const CREAM_PANEL: Color = Color::new(1.00, 0.96, 0.88, 0.88);
const PANEL_EDGE: Color = Color::new(0.78, 0.62, 0.40, 0.7);

fn player_color(idx: usize) -> Color {
    match idx {
        0 => ORANGE,
        1 => GREEN,
        2 => PURPLE,
        3 => CYAN,
        _ => WHITE,
    }
}

fn player_texture<'a>(idx: usize, assets: &'a AssetHandles) -> &'a Texture2D {
    match idx {
        0 => &assets.edie_static_run,
        1 => &assets.obstacle_alice3,
        2 => &assets.obstacle_amy,
        3 => &assets.obstacle_boxbot,
        _ => &assets.edie_static_run,
    }
}

fn player_name(idx: usize) -> &'static str {
    match idx {
        0 => "EDIE",
        1 => "ALICE",
        2 => "AMY",
        3 => "BOXBOT",
        _ => "???",
    }
}

/// Player name accounting for the TEIO easter-egg swap on slot 0.
fn effective_player_name(idx: usize, teio: bool) -> &'static str {
    if teio && idx == 0 { "TEIO" } else { player_name(idx) }
}

fn teio_gold() -> Color { Color::new(1.00, 0.85, 0.30, 1.0) }

/// Player color accounting for TEIO swap.
fn effective_player_color(idx: usize, teio: bool) -> Color {
    if teio && idx == 0 { teio_gold() } else { player_color(idx) }
}

/// Anchor for each player's home zone in logical coords (top-left of the
/// 4×2 piece grid).
fn home_anchor(pi: usize) -> (f32, f32) {
    match pi {
        0 => (40.0,  560.0),   // bottom-left
        1 => (1090.0, 560.0),  // bottom-right
        2 => (40.0,  40.0),    // top-left
        3 => (1090.0, 40.0),   // top-right
        _ => (40.0, 560.0),
    }
}

/// Grid offset for piece `qi` within a home zone.
fn home_piece_offset(qi: usize) -> (f32, f32) {
    let col = (qi % 2) as f32;
    let row = (qi / 2) as f32;
    (col * 54.0, row * 54.0)
}

fn home_piece_center(pi: usize, qi: usize) -> (f32, f32) {
    let (ax, ay) = home_anchor(pi);
    let (ox, oy) = home_piece_offset(qi);
    (ax + ox + 24.0, ay + oy + 24.0)
}

/// Logical position of each board cell for rendering.
fn cell_pos(pos: usize) -> (f32, f32) {
    let cx = 640.0;
    let cy = 360.0;
    let sp = 52.0; // spacing between cells
    match pos {
        // Bottom edge (right to left): 0=start, 1..4
        0 => (cx + sp * 2.5, cy + sp * 2.5),
        1 => (cx + sp * 1.5, cy + sp * 2.5),
        2 => (cx + sp * 0.5, cy + sp * 2.5),
        3 => (cx - sp * 0.5, cy + sp * 2.5),
        4 => (cx - sp * 1.5, cy + sp * 2.5),
        // Right edge (bottom to top): 5=NE corner, 6..9
        5 => (cx - sp * 2.5, cy + sp * 2.5),
        6 => (cx - sp * 2.5, cy + sp * 1.5),
        7 => (cx - sp * 2.5, cy + sp * 0.5),
        8 => (cx - sp * 2.5, cy - sp * 0.5),
        9 => (cx - sp * 2.5, cy - sp * 1.5),
        // Top edge (left to right): 10=NW corner, 11..14
        10 => (cx - sp * 2.5, cy - sp * 2.5),
        11 => (cx - sp * 1.5, cy - sp * 2.5),
        12 => (cx - sp * 0.5, cy - sp * 2.5),
        13 => (cx + sp * 0.5, cy - sp * 2.5),
        14 => (cx + sp * 1.5, cy - sp * 2.5),
        // Left edge (top to bottom): 15=SW corner, 16..19
        15 => (cx + sp * 2.5, cy - sp * 2.5),
        16 => (cx + sp * 2.5, cy - sp * 1.5),
        17 => (cx + sp * 2.5, cy - sp * 0.5),
        18 => (cx + sp * 2.5, cy + sp * 0.5),
        19 => (cx + sp * 2.5, cy + sp * 1.5),
        // Diagonal A: 5→20→21→24→27→28
        20 => (cx - sp * 1.7, cy + sp * 1.7),
        21 => (cx - sp * 0.85, cy + sp * 0.85),
        // Diagonal B: 10→22→23→24→25→26
        22 => (cx - sp * 1.7, cy - sp * 1.7),
        23 => (cx - sp * 0.85, cy - sp * 0.85),
        // Center
        24 => (cx, cy),
        // From center toward exit
        25 => (cx + sp * 0.85, cy + sp * 0.85),
        26 => (cx + sp * 1.7, cy + sp * 1.7),
        27 => (cx + sp * 0.85, cy - sp * 0.85),
        28 => (cx + sp * 1.7, cy - sp * 1.7),
        _ => (0.0, 0.0),
    }
}

pub fn draw_yut(game: &YutGame, assets: &AssetHandles, elapsed: f32) {
    let cam = Camera::with_logical(YUT_W, YUT_H, screen_width(), screen_height());
    clear_background(Color::new(0.96, 0.94, 0.88, 1.0));
    draw_daylight_backdrop(elapsed, &cam);

    match game.phase {
        Phase::Menu => draw_menu(game, assets, elapsed, &cam),
        Phase::GameOver => {
            draw_board(&cam);
            draw_home_zones(game, assets, elapsed, &cam);
            draw_all_pieces(game, assets, elapsed, &cam);
            draw_game_over(game, assets, elapsed, &cam);
        }
        _ => {
            draw_board(&cam);
            draw_home_zones(game, assets, elapsed, &cam);
            draw_all_pieces(game, assets, elapsed, &cam);
            draw_hud(game, assets, &cam);
            draw_power_cards(game, &cam);
            draw_throw_result(game, elapsed, &cam);
            if game.phase == Phase::SelectPiece {
                draw_piece_selection_hint(game, elapsed, &cam);
            }
            if game.phase == Phase::SelectPath {
                draw_path_choice(game, &cam);
            }
            // Draw traps and blocked cells
            draw_traps_and_blocks(game, elapsed, &cam);
            if let Some((ref msg, t)) = game.toast {
                draw_toast(msg, t, &cam);
            }
        }
    }
}

fn draw_daylight_backdrop(elapsed: f32, cam: &Camera) {
    let (x0, y0) = cam.to_screen(0.0, 0.0);
    let w = cam.scaled(YUT_W);
    let h = cam.scaled(YUT_H);
    let t = (elapsed * 0.3).sin() * 0.5 + 0.5;
    let bg_top = Color::new(1.00, 0.90 + 0.03 * t, 0.80, 1.0);
    let bg_bot = Color::new(0.86, 0.96, 0.90 + 0.02 * t, 1.0);
    draw_rectangle(x0, y0, w, h * 0.5, bg_top);
    draw_rectangle(x0, y0 + h * 0.5, w, h * 0.5, bg_bot);
    // Soft corner glows
    for corner in &[(60.0, 60.0), (YUT_W - 60.0, 60.0), (60.0, YUT_H - 60.0), (YUT_W - 60.0, YUT_H - 60.0)] {
        let (sx, sy) = cam.to_screen(corner.0, corner.1);
        let pulse = 0.25 + 0.15 * (elapsed * 1.4 + corner.0 * 0.01).sin();
        draw_circle(sx, sy, cam.scaled(24.0), Color::new(0.98, 0.66, 0.32, pulse * 0.35));
        draw_circle(sx, sy, cam.scaled(14.0), Color::new(0.46, 0.95, 0.74, pulse * 0.28));
    }
}

fn draw_board(cam: &Camera) {
    // Board backdrop (subtle panel under the cross)
    let (bx, by) = cam.to_screen(640.0 - 180.0, 360.0 - 180.0);
    draw_rectangle(bx, by, cam.scaled(360.0), cam.scaled(360.0), Color::new(1.0, 0.96, 0.88, 0.55));
    draw_rectangle_lines(bx, by, cam.scaled(360.0), cam.scaled(360.0), 2.0, PANEL_EDGE);

    // Draw connections
    let edges: &[(usize, usize)] = &[
        (0,1),(1,2),(2,3),(3,4),(4,5),
        (5,6),(6,7),(7,8),(8,9),(9,10),
        (10,11),(11,12),(12,13),(13,14),(14,15),
        (15,16),(16,17),(17,18),(18,19),(19,0),
        // Diagonal A
        (5,20),(20,21),(21,24),(24,27),(27,28),
        // Diagonal B
        (10,22),(22,23),(23,24),(24,25),(25,26),
    ];
    for &(a, b) in edges {
        let (ax, ay) = cell_pos(a);
        let (bx, by) = cell_pos(b);
        let (sax, say) = cam.to_screen(ax, ay);
        let (sbx, sby) = cam.to_screen(bx, by);
        draw_line(sax, say, sbx, sby, 2.0, Color::new(0.55, 0.42, 0.22, 0.55));
    }
    // Draw cells
    for pos in 0..NUM_POSITIONS {
        let (lx, ly) = cell_pos(pos);
        let (sx, sy) = cam.to_screen(lx, ly);
        let r = cam.scaled(if is_shortcut_corner(pos) { 16.0 } else if pos == CENTER { 18.0 } else { 12.0 });
        let col = if pos == CENTER {
            Color::new(0.46, 0.95, 0.74, 0.85)
        } else if is_shortcut_corner(pos) {
            Color::new(0.98, 0.85, 0.55, 0.9)
        } else if pos == 0 {
            Color::new(0.98, 0.66, 0.32, 0.9)
        } else {
            Color::new(0.99, 0.94, 0.84, 0.95)
        };
        draw_circle(sx, sy, r, col);
        draw_circle_lines(sx, sy, r, 2.0, Color::new(0.55, 0.42, 0.22, 0.8));
    }
    // Labels
    let labels = [("START", 0), ("NE", 5), ("NW", 10), ("SW", 15)];
    for (label, pos) in labels {
        let (lx, ly) = cell_pos(pos);
        let size = 11.0 * cam.scale;
        let dim = measure_text(label, None, size as u16, 1.0);
        let (sx, sy) = cam.to_screen(lx, ly - 22.0);
        draw_text(label, sx - dim.width * 0.5, sy, size, Color::new(0.35, 0.25, 0.12, 0.85));
    }
}

fn draw_piece_sprite(assets: &AssetHandles, pi: usize, sx: f32, sy: f32, logical_size: f32, cam: &Camera, stack: u8, shield: u8, teio: bool, elapsed: f32) {
    let tex = player_texture(pi, assets);
    let color = effective_player_color(pi, teio);
    // Colored halo behind the sprite so each player stays distinguishable
    // even with shared-style character art.
    let halo = cam.scaled(logical_size * 0.72);
    draw_circle(sx, sy, halo, Color::new(color.r, color.g, color.b, 0.85));
    draw_circle_lines(sx, sy, halo, 1.5, Color::new(0.20, 0.12, 0.06, 0.7));
    let src_w = tex.width();
    let src_h = tex.height();
    let fit = (logical_size / src_w).min(logical_size / src_h);
    let pw = src_w * fit;
    let ph = src_h * fit;
    let (dx, dy) = (sx - cam.scaled(pw) * 0.5, sy - cam.scaled(ph) * 0.5);
    // TEIO slot 0 gets a gold wash on the sprite; others render as-is.
    let sprite_tint = if teio && pi == 0 {
        Color::new(1.0, 0.88, 0.55, 1.0)
    } else { WHITE };
    draw_texture_ex(tex, dx, dy, sprite_tint, DrawTextureParams {
        dest_size: Some(vec2(cam.scaled(pw), cam.scaled(ph))),
        ..Default::default()
    });
    if teio && pi == 0 {
        draw_teio_sparkles(sx, sy, halo, elapsed, cam);
    }
    if stack > 1 {
        let txt = format!("x{}", stack);
        let ts = 13.0 * cam.scale;
        let td = measure_text(&txt, None, ts as u16, 1.0);
        let bx = sx + halo - cam.scaled(4.0);
        let by = sy + halo - cam.scaled(4.0);
        draw_rectangle(bx - td.width - cam.scaled(4.0), by - td.height - cam.scaled(2.0),
            td.width + cam.scaled(8.0), td.height + cam.scaled(4.0),
            Color::new(0.20, 0.12, 0.06, 0.85));
        draw_text(&txt, bx - td.width, by, ts, Color::new(1.0, 0.95, 0.75, 1.0));
    }
    if shield > 0 {
        draw_circle_lines(sx, sy, halo + cam.scaled(4.0), 2.5,
            Color::new(0.3, 0.9, 1.0, 0.9));
    }
}

/// Four small 4-pointed stars orbiting a TEIO piece.
fn draw_teio_sparkles(cx: f32, cy: f32, halo: f32, elapsed: f32, cam: &Camera) {
    let gold = Color::new(1.0, 0.92, 0.45, 0.9);
    let ring = halo + cam.scaled(8.0);
    for i in 0..4 {
        let phase = elapsed * 1.6 + i as f32 * std::f32::consts::FRAC_PI_2;
        let sx = cx + phase.cos() * ring;
        let sy = cy + phase.sin() * ring;
        let arm = cam.scaled(4.0);
        draw_line(sx - arm, sy, sx + arm, sy, 1.5, gold);
        draw_line(sx, sy - arm, sx, sy + arm, 1.5, gold);
    }
}

fn draw_all_pieces(game: &YutGame, assets: &AssetHandles, elapsed: f32, cam: &Camera) {
    for (pi, player) in game.players.iter().enumerate() {
        for (qi, piece) in player.pieces.iter().enumerate() {
            if piece.is_exited() || piece.stack == 0 { continue; }
            if piece.is_home() {
                let (hx, hy) = home_piece_center(pi, qi);
                let (sx, sy) = cam.to_screen(hx, hy);
                draw_piece_sprite(assets, pi, sx, sy, 34.0, cam, 1, 0, game.teio_unlocked, elapsed);
            } else {
                let (lx, ly) = cell_pos(piece.pos);
                let offset = qi as f32 * 5.0 - 7.5;
                let bob = (elapsed * 2.5 + pi as f32 + qi as f32).sin() * 2.0;
                let (sx, sy) = cam.to_screen(lx + offset, ly + bob);
                let size = if piece.stack > 1 { 32.0 } else { 26.0 };
                draw_piece_sprite(assets, pi, sx, sy, size, cam, piece.stack, piece.shield, game.teio_unlocked, elapsed);
            }
        }
    }
}

fn draw_home_zones(game: &YutGame, assets: &AssetHandles, elapsed: f32, cam: &Camera) {
    for pi in 0..game.num_players {
        let (ax, ay) = home_anchor(pi);
        let (sx, sy) = cam.to_screen(ax - 8.0, ay - 32.0);
        let w = cam.scaled(148.0);
        let h = cam.scaled(148.0);
        let active = pi == game.current_player && !matches!(game.phase, Phase::Menu | Phase::GameOver);
        let panel = if active {
            let pulse = 0.2 + 0.1 * (elapsed * 3.0).sin();
            Color::new(CREAM_PANEL.r, CREAM_PANEL.g, CREAM_PANEL.b, 0.88 + pulse)
        } else {
            CREAM_PANEL
        };
        draw_rectangle(sx, sy, w, h, panel);
        draw_rectangle_lines(sx, sy, w, h, 2.0, PANEL_EDGE);
        // Player label — switches to TEIO once the egg is unlocked for slot 0.
        let name = effective_player_name(pi, game.teio_unlocked);
        let ns = 14.0 * cam.scale;
        let nd = measure_text(name, None, ns as u16, 1.0);
        let label_col = effective_player_color(pi, game.teio_unlocked);
        draw_text(name, sx + (w - nd.width) * 0.5, sy + cam.scaled(18.0), ns,
            Color::new(label_col.r * 0.75, label_col.g * 0.55, label_col.b * 0.45, 1.0));
        // Count summary
        let player = &game.players.get(pi);
        if let Some(p) = player {
            let home = p.pieces.iter().filter(|q| q.is_home()).count();
            let board = p.pieces.iter().filter(|q| q.is_on_board() && q.stack > 0).count();
            let done = p.pieces.iter().filter(|q| q.is_exited()).count();
            let txt = format!("H{} B{} D{}", home, board, done);
            let ts = 12.0 * cam.scale;
            let td = measure_text(&txt, None, ts as u16, 1.0);
            draw_text(&txt, sx + (w - td.width) * 0.5, sy + h - cam.scaled(8.0), ts, WARM_TEXT);
        }
        // Empty slot ghosts for missing players — skip
        let _ = assets;
    }
}

fn draw_hud(game: &YutGame, _assets: &AssetHandles, cam: &Camera) {
    // Top banner pill with turn / player
    let current = effective_player_name(game.current_player, game.teio_unlocked);
    let turn_txt = format!("Turn {} — {}'s turn", game.turn_count + 1, current);
    let size = 22.0 * cam.scale;
    let dim = measure_text(&turn_txt, None, size as u16, 1.0);
    let (tx, ty) = cam.to_screen(640.0, 28.0);
    let px = cam.scaled(14.0);
    let py = cam.scaled(6.0);
    draw_rectangle(tx - dim.width * 0.5 - px, ty - dim.height - py,
        dim.width + px * 2.0, dim.height + py * 2.0,
        CREAM_PANEL);
    draw_rectangle_lines(tx - dim.width * 0.5 - px, ty - dim.height - py,
        dim.width + px * 2.0, dim.height + py * 2.0, 1.5, PANEL_EDGE);
    let c = effective_player_color(game.current_player, game.teio_unlocked);
    let dark = Color::new(c.r * 0.7, c.g * 0.55, c.b * 0.45, 1.0);
    draw_text(&turn_txt, tx - dim.width * 0.5, ty, size, dark);

    // Phase hint near the bottom of the board
    let hint = match game.phase {
        Phase::Throwing => "TAP / SPACE to throw yut",
        Phase::SelectPiece => "Select a piece to move (tap or 1-4)",
        Phase::SelectPath => "Choose path: [1] Shortcut  [2] Outer",
        _ => "",
    };
    if !hint.is_empty() {
        let hs = 17.0 * cam.scale;
        let hd = measure_text(hint, None, hs as u16, 1.0);
        let (hx, hy) = cam.to_screen(640.0, 702.0);
        draw_rectangle(hx - hd.width * 0.5 - cam.scaled(10.0), hy - hd.height - cam.scaled(4.0),
            hd.width + cam.scaled(20.0), hd.height + cam.scaled(8.0), CREAM_PANEL);
        draw_text(hint, hx - hd.width * 0.5, hy, hs, WARM_TEXT);
    }
}

fn draw_throw_result(game: &YutGame, elapsed: f32, cam: &Camera) {
    // Central tray between the bottom home zones (x≈200..1080, y≈560..720)
    let tray_x = 260.0;
    let tray_y = 560.0;
    let tray_w = 760.0;
    let tray_h = 150.0;
    let (tx, ty) = cam.to_screen(tray_x, tray_y);
    draw_rectangle(tx, ty, cam.scaled(tray_w), cam.scaled(tray_h),
        Color::new(1.0, 0.95, 0.86, 0.82));
    draw_rectangle_lines(tx, ty, cam.scaled(tray_w), cam.scaled(tray_h), 2.0, PANEL_EDGE);

    if let Some(result) = game.last_throw {
        let txt = format!("{} {}  ({}칸)", result.name_ko(), result.name_en(), result.steps());
        let size = 34.0 * cam.scale;
        let dim = measure_text(&txt, None, size as u16, 1.0);
        let (lx, ly) = cam.to_screen(tray_x + tray_w * 0.5, tray_y + 38.0);
        let tint = if result.grants_bonus() {
            Color::new(0.95, 0.45, 0.15, 1.0)
        } else {
            Color::new(0.55, 0.35, 0.18, 1.0)
        };
        draw_text(&txt, lx - dim.width * 0.5, ly, size, tint);

        if let Some(sticks) = game.last_sticks {
            // Draw the 4 yut sticks as rounded pills; flat = pale wood + dot,
            // round = darker wood with a full curve.
            let stick_w = 86.0;
            let stick_h = 22.0;
            let gap = 14.0;
            let total = stick_w * 4.0 + gap * 3.0;
            let start_x = tray_x + (tray_w - total) * 0.5;
            let stick_y = tray_y + 82.0;
            for (i, &flat) in sticks.iter().enumerate() {
                let lx = start_x + i as f32 * (stick_w + gap);
                let wiggle = if result == crate::yut::throw::YutResult::Mo || result == crate::yut::throw::YutResult::Yut {
                    (elapsed * 6.0 + i as f32).sin() * 2.0
                } else { 0.0 };
                let (sx, sy) = cam.to_screen(lx, stick_y + wiggle);
                let w = cam.scaled(stick_w);
                let h = cam.scaled(stick_h);
                let (fill, edge) = if flat {
                    (Color::new(0.98, 0.90, 0.70, 1.0), Color::new(0.65, 0.45, 0.22, 0.95))
                } else {
                    (Color::new(0.55, 0.36, 0.20, 1.0), Color::new(0.30, 0.18, 0.08, 0.95))
                };
                draw_rectangle(sx, sy, w, h, fill);
                draw_rectangle_lines(sx, sy, w, h, 2.0, edge);
                // Flat side marker: a small dark dot at center
                if flat {
                    draw_circle(sx + w * 0.5, sy + h * 0.5, cam.scaled(2.5),
                        Color::new(0.30, 0.18, 0.08, 0.9));
                }
            }
        }
    } else {
        let hint = "눌러서 윷 던지기 / TAP to throw!";
        let size = 22.0 * cam.scale;
        let dim = measure_text(hint, None, size as u16, 1.0);
        let (lx, ly) = cam.to_screen(tray_x + tray_w * 0.5, tray_y + tray_h * 0.5 + 10.0);
        let pulse = 0.6 + 0.4 * (elapsed * 3.5).sin().abs();
        draw_text(hint, lx - dim.width * 0.5, ly, size,
            Color::new(WARM_TEXT.r, WARM_TEXT.g, WARM_TEXT.b, pulse));
    }
}

fn draw_piece_selection_hint(game: &YutGame, elapsed: f32, cam: &Camera) {
    let pi = game.current_player;
    let player = &game.players[pi];
    let pulse = 0.5 + 0.5 * (elapsed * 5.0).sin();
    let highlight = Color::new(1.0, 0.85, 0.25, pulse);
    for (i, piece) in player.pieces.iter().enumerate() {
        if piece.is_exited() || piece.stack == 0 { continue; }
        if piece.is_home() {
            let (hx, hy) = home_piece_center(pi, i);
            let (sx, sy) = cam.to_screen(hx, hy);
            draw_circle_lines(sx, sy, cam.scaled(22.0), 3.0, highlight);
        } else {
            let (lx, ly) = cell_pos(piece.pos);
            let offset = i as f32 * 5.0 - 7.5;
            let (sx, sy) = cam.to_screen(lx + offset, ly);
            draw_circle_lines(sx, sy, cam.scaled(18.0), 3.0, highlight);
        }
    }
}

fn draw_path_choice(_game: &YutGame, cam: &Camera) {
    let (x0, y0) = cam.to_screen(0.0, 0.0);
    draw_rectangle(x0, y0, cam.scaled(YUT_W), cam.scaled(YUT_H),
        Color::new(1.0, 0.96, 0.88, 0.55));
    let (px, py) = cam.to_screen(390.0, 280.0);
    let pw = cam.scaled(500.0);
    let ph = cam.scaled(150.0);
    draw_rectangle(px, py, pw, ph, CREAM_PANEL);
    draw_rectangle_lines(px, py, pw, ph, 2.0, PANEL_EDGE);
    let opts = ["[1] SHORTCUT (diagonal)", "[2] OUTER PATH (around)"];
    let tints = [Color::new(0.25, 0.60, 0.42, 1.0), Color::new(0.78, 0.52, 0.20, 1.0)];
    for (i, opt) in opts.iter().enumerate() {
        let size = 26.0 * cam.scale;
        let dim = measure_text(opt, None, size as u16, 1.0);
        let (tx, ty) = cam.to_screen(640.0, 320.0 + i as f32 * 60.0);
        draw_text(opt, tx - dim.width * 0.5, ty, size, tints[i]);
    }
}

fn draw_menu(game: &YutGame, assets: &AssetHandles, elapsed: f32, cam: &Camera) {
    // AeiROBOT brand line above the title.
    let brand = "AeiROBOT × EDIE";
    let bs = 18.0 * cam.scale;
    let bd = measure_text(brand, None, bs as u16, 1.0);
    let (bx, by) = cam.to_screen(640.0, 150.0);
    draw_text(brand, bx - bd.width * 0.5, by, bs, Color::new(0.45, 0.28, 0.08, 0.85));

    let title = "EDIE YUT NORI";
    let sub = "초능력 윷놀이";
    let size = 54.0 * cam.scale;
    let dim = measure_text(title, None, size as u16, 1.0);
    let (tx, ty) = cam.to_screen(640.0, 200.0);
    draw_text(title, tx - dim.width * 0.5 + 3.0, ty + 3.0, size, Color::new(1.0, 0.97, 0.85, 0.6));
    draw_text(title, tx - dim.width * 0.5, ty, size, Color::new(0.95, 0.55, 0.15, 1.0));
    let ss = 24.0 * cam.scale;
    let sd = measure_text(sub, None, ss as u16, 1.0);
    let (sxp, syp) = cam.to_screen(640.0, 246.0);
    draw_text(sub, sxp - sd.width * 0.5, syp, ss, Color::new(0.25, 0.62, 0.46, 1.0));

    if game.teio_unlocked {
        let teio = "★ TEIO MODE ★";
        let ts = 16.0 * cam.scale;
        let td = measure_text(teio, None, ts as u16, 1.0);
        let (tx2, ty2) = cam.to_screen(640.0, 272.0);
        draw_text(teio, tx2 - td.width * 0.5, ty2, ts, teio_gold());
    }

    // Character cameo row
    let cameos = [&assets.edie_static_run, &assets.obstacle_alice3, &assets.obstacle_amy, &assets.obstacle_boxbot];
    let colors = [
        effective_player_color(0, game.teio_unlocked),
        player_color(1),
        player_color(2),
        player_color(3),
    ];
    let start = 640.0 - (cameos.len() as f32 - 1.0) * 0.5 * 96.0;
    for (i, tex) in cameos.iter().enumerate() {
        let lx = start + i as f32 * 96.0;
        let ly = 300.0 + ((elapsed * 2.5 + i as f32 * 0.7).sin() * 4.0);
        let (sx, sy) = cam.to_screen(lx, ly);
        let halo = cam.scaled(30.0);
        draw_circle(sx, sy, halo, Color::new(colors[i].r, colors[i].g, colors[i].b, 0.85));
        draw_circle_lines(sx, sy, halo, 2.0, PANEL_EDGE);
        let src_w = tex.width();
        let src_h = tex.height();
        let fit = (44.0 / src_w).min(44.0 / src_h);
        let pw = src_w * fit;
        let ph = src_h * fit;
        let tint = if i == 0 && game.teio_unlocked { Color::new(1.0, 0.88, 0.55, 1.0) } else { WHITE };
        draw_texture_ex(tex, sx - cam.scaled(pw) * 0.5, sy - cam.scaled(ph) * 0.5, tint, DrawTextureParams {
            dest_size: Some(vec2(cam.scaled(pw), cam.scaled(ph))),
            ..Default::default()
        });
        if i == 0 && game.teio_unlocked {
            draw_teio_sparkles(sx, sy, halo, elapsed, cam);
        }
    }

    let opts = [("1. 2P GAME", 400.0), ("2. 3P GAME", 450.0), ("3. 4P GAME", 500.0)];
    let os = 26.0 * cam.scale;
    for (l, y) in &opts {
        let d = measure_text(l, None, os as u16, 1.0);
        let (ox, oy) = cam.to_screen(640.0, *y);
        // Option pill
        let (px, py) = cam.to_screen(640.0 - 150.0, y - 30.0);
        let pw = cam.scaled(300.0);
        let ph = cam.scaled(40.0);
        draw_rectangle(px, py, pw, ph, CREAM_PANEL);
        draw_rectangle_lines(px, py, pw, ph, 1.5, PANEL_EDGE);
        draw_text(l, ox - d.width * 0.5, oy, os, Color::new(0.22, 0.58, 0.42, 1.0));
    }
    let hint = "CLICK OR PRESS 1-3";
    let hs = 16.0 * cam.scale;
    let hd = measure_text(hint, None, hs as u16, 1.0);
    let (hx, hy) = cam.to_screen(640.0, 570.0);
    draw_text(hint, hx - hd.width * 0.5, hy, hs, Color::new(0.35, 0.25, 0.12, 0.85));
}

fn draw_game_over(game: &YutGame, assets: &AssetHandles, elapsed: f32, cam: &Camera) {
    let (x0, y0) = cam.to_screen(0.0, 0.0);
    draw_rectangle(x0, y0, cam.scaled(YUT_W), cam.scaled(YUT_H),
        Color::new(1.0, 0.97, 0.90, 0.78));
    let winner_idx = game.winner.unwrap_or(0);
    let winner_name = effective_player_name(winner_idx, game.teio_unlocked);
    let title = format!("{} WINS!", winner_name);
    let c = game.winner.map(|w| effective_player_color(w, game.teio_unlocked)).unwrap_or(ORANGE);
    let color = Color::new(c.r * 0.75, c.g * 0.55, c.b * 0.4, 1.0);
    let size = 58.0 * cam.scale;
    let dim = measure_text(&title, None, size as u16, 1.0);
    let (tx, ty) = cam.to_screen(640.0, 280.0);
    draw_text(&title, tx - dim.width * 0.5 + 3.0, ty + 3.0, size, Color::new(1.0, 0.95, 0.82, 0.7));
    draw_text(&title, tx - dim.width * 0.5, ty, size, color);

    // Winner character cameo
    if let Some(w) = game.winner {
        let tex = player_texture(w, assets);
        let bounce = ((elapsed * 5.0).sin() * 8.0).abs();
        let (wx, wy) = cam.to_screen(640.0, 380.0 - bounce);
        let src_w = tex.width();
        let src_h = tex.height();
        let fit = (140.0 / src_w).min(140.0 / src_h);
        let pw = src_w * fit;
        let ph = src_h * fit;
        draw_circle(wx, wy, cam.scaled(80.0), Color::new(c.r, c.g, c.b, 0.55));
        draw_texture_ex(tex, wx - cam.scaled(pw) * 0.5, wy - cam.scaled(ph) * 0.5, WHITE, DrawTextureParams {
            dest_size: Some(vec2(cam.scaled(pw), cam.scaled(ph))),
            ..Default::default()
        });
    }

    let turns = format!("in {} turns", game.turn_count);
    let ts = 22.0 * cam.scale;
    let td = measure_text(&turns, None, ts as u16, 1.0);
    let (ttx, tty) = cam.to_screen(640.0, 500.0);
    draw_text(&turns, ttx - td.width * 0.5, tty, ts, WARM_TEXT);
    let sub = "TAP or SPACE to play again";
    let ss = 18.0 * cam.scale;
    let sd = measure_text(sub, None, ss as u16, 1.0);
    let (sx, sy) = cam.to_screen(640.0, 540.0);
    draw_text(sub, sx - sd.width * 0.5, sy, ss, Color::new(0.35, 0.25, 0.12, 0.85));
}

fn draw_power_cards(game: &YutGame, cam: &Camera) {
    let pi = game.current_player;
    let cards = &game.power_cards[pi];
    if cards.is_empty() { return; }
    // Panel below the top HUD on the right-hand side
    let panel_x = 840.0;
    let panel_y = 80.0;
    let panel_w = 330.0;
    let panel_h = 24.0 + cards.len() as f32 * 28.0 + 6.0;
    let (px, py) = cam.to_screen(panel_x, panel_y);
    draw_rectangle(px, py, cam.scaled(panel_w), cam.scaled(panel_h), CREAM_PANEL);
    draw_rectangle_lines(px, py, cam.scaled(panel_w), cam.scaled(panel_h), 1.5, PANEL_EDGE);
    let ts = 13.0 * cam.scale;
    let header = format!("{} 초능력", effective_player_name(pi, game.teio_unlocked));
    let (hx, hy) = cam.to_screen(panel_x + 12.0, panel_y + 20.0);
    draw_text(&header, hx, hy, ts, WARM_TEXT);
    for (i, card) in cards.iter().enumerate() {
        let y = panel_y + 40.0 + i as f32 * 28.0;
        let (cx, cy) = cam.to_screen(panel_x + 8.0, y);
        let cw = cam.scaled(panel_w - 16.0);
        let ch = cam.scaled(22.0);
        draw_rectangle(cx, cy - ch * 0.5, cw, ch, Color::new(0.98, 0.88, 0.70, 0.95));
        draw_rectangle_lines(cx, cy - ch * 0.5, cw, ch, 1.0, PANEL_EDGE);
        let key = if i == 0 { "[Q]" } else { "[W]" };
        let label = format!("{} {} — {}", key, card.name(), card.desc());
        let (lx, ly) = cam.to_screen(panel_x + 16.0, y + 4.0);
        draw_text(&label, lx, ly, ts, Color::new(0.45, 0.28, 0.08, 1.0));
    }
}

fn draw_traps_and_blocks(game: &YutGame, elapsed: f32, cam: &Camera) {
    let pulse = 0.5 + 0.5 * (elapsed * 4.0).sin();
    // Draw traps (warm red X on the cell)
    for &(pos, _owner) in &game.traps {
        let (lx, ly) = cell_pos(pos);
        let (sx, sy) = cam.to_screen(lx, ly);
        let r = cam.scaled(10.0);
        let col = Color::new(0.92, 0.28, 0.22, pulse);
        draw_line(sx - r, sy - r, sx + r, sy + r, 2.5, col);
        draw_line(sx + r, sy - r, sx - r, sy + r, 2.5, col);
    }
    // Draw blocked cells (soft purple halo)
    for &(pos, turns) in &game.blocked_cells {
        let (lx, ly) = cell_pos(pos);
        let (sx, sy) = cam.to_screen(lx, ly);
        draw_circle(sx, sy, cam.scaled(14.0), Color::new(0.72, 0.56, 0.92, 0.55 * pulse));
        let txt = format!("{}", turns);
        let ts = 11.0 * cam.scale;
        let td = measure_text(&txt, None, ts as u16, 1.0);
        draw_text(&txt, sx - td.width * 0.5, sy + td.height * 0.3, ts, WARM_TEXT);
    }
}

fn draw_toast(msg: &str, remaining: f32, cam: &Camera) {
    let alpha = remaining.min(1.0);
    let size = 26.0 * cam.scale;
    let dim = measure_text(msg, None, size as u16, 1.0);
    let (tx, ty) = cam.to_screen(640.0, 80.0);
    let px = cam.scaled(14.0);
    let py = cam.scaled(6.0);
    draw_rectangle(tx - dim.width * 0.5 - px, ty - dim.height - py,
        dim.width + px * 2.0, dim.height + py * 2.0,
        Color::new(CREAM_PANEL.r, CREAM_PANEL.g, CREAM_PANEL.b, 0.9 * alpha));
    draw_rectangle_lines(tx - dim.width * 0.5 - px, ty - dim.height - py,
        dim.width + px * 2.0, dim.height + py * 2.0, 1.5,
        Color::new(PANEL_EDGE.r, PANEL_EDGE.g, PANEL_EDGE.b, alpha));
    draw_text(msg, tx - dim.width * 0.5, ty, size, Color::new(0.75, 0.40, 0.10, alpha));
}

/// Convert screen coords to board cell index (for touch/click).
pub fn screen_to_board_cell(screen_x: f32, screen_y: f32) -> Option<usize> {
    let cam = Camera::with_logical(YUT_W, YUT_H, screen_width(), screen_height());
    let lx = (screen_x - cam.offset_x) / cam.scale;
    let ly = (screen_y - cam.offset_y) / cam.scale;
    let threshold = 24.0;
    for pos in 0..NUM_POSITIONS {
        let (cx, cy) = cell_pos(pos);
        let dx = lx - cx;
        let dy = ly - cy;
        if dx * dx + dy * dy < threshold * threshold {
            return Some(pos);
        }
    }
    None
}

/// Check if screen coords hit a player's home piece.
pub fn screen_to_home_piece(screen_x: f32, screen_y: f32, game: &YutGame) -> Option<usize> {
    let cam = Camera::with_logical(YUT_W, YUT_H, screen_width(), screen_height());
    let lx = (screen_x - cam.offset_x) / cam.scale;
    let ly = (screen_y - cam.offset_y) / cam.scale;
    let pi = game.current_player;
    for (qi, piece) in game.players[pi].pieces.iter().enumerate() {
        if !piece.is_home() || piece.stack == 0 { continue; }
        let (hx, hy) = home_piece_center(pi, qi);
        let dx = lx - hx;
        let dy = ly - hy;
        if dx * dx + dy * dy < 22.0 * 22.0 {
            return Some(qi);
        }
    }
    None
}
