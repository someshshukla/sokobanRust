use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use sdl2::video::Window;

use game::{Direction, Level, Position};
use shadow::ShadowFlags;
use tileset::{Tile, Tileset, TilesetSelector};

/// The Painter struct is responsible for drawing the game onto the screen.
pub struct Painter<'a> {
    /// The tileset selector
    selector: TilesetSelector<'a>,
    /// The font used to display text
    font: Font<'a, 'a>,
    /// The size of the screen in pixels
    screen_size: (u32, u32),
    /// The height of the status bar
    bar_height: u32,
    /// The color of the status bar
    bar_color: Color,
    /// The color of the text in the status bar
    bar_text_color: Color,
}

/// Represents a location for text in the status bar
#[derive(Clone, Copy)]
enum StatusBarLocation {
    FlushLeft,
    FlushRight,
}

impl<'a> Painter<'a> {
    /// Creates a new instance.
    pub fn new(
        canvas: &mut Canvas<Window>,
        big_set: Tileset<'a>,
        small_set: Tileset<'a>,
        font: Font<'a, 'a>,
    ) -> Painter<'a> {
        let screen_size = canvas.window().drawable_size();
        let selector = TilesetSelector::new(big_set, small_set);
        Painter {
            selector,
            font,
            screen_size,
            bar_height: 32,
            bar_color: Color::RGBA(20, 20, 20, 255),
            bar_text_color: Color::RGBA(255, 192, 0, 255),
        }
    }

    /// Paints a level onto the screen.
    pub fn paint(&mut self, canvas: &mut Canvas<Window>, level: &Level) {
        self.selector.reset(level.extents());

        // Draw a full-size image onto an off-screen buffer
        let fullsize = self.tileset().get_rendering_size(level.extents());
        let creator = canvas.texture_creator();
        let mut texture = creator
            .create_texture_target(PixelFormatEnum::RGBA8888, fullsize.0, fullsize.1)
            .expect("Could not get texture target for off-screen rendering");

        canvas
            .with_texture_canvas(&mut texture, |cv| {
                self.paint_fullsize(cv, level);
            })
            .unwrap();

        // Copy onto the screen with appropriate scaling
        let final_rect = self.get_centered_image_rect(self.get_scaled_rendering_size(&level));

        canvas.clear();
        let original_rect = Some(Rect::new(0, 0, fullsize.0, fullsize.1));
        canvas.copy(&texture, original_rect, final_rect).unwrap();

        self.paint_status_bar(canvas, &level);

        canvas.present();
    }

    /// Paints a full-size image of the given level onto the current render target.
    fn paint_fullsize(&mut self, canvas: &mut Canvas<Window>, level: &Level) {
        let (cols, rows) = level.extents();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        for r in 0..rows {
            for c in 0..cols {
                let pos = Position::new(r, c);
                let (x, y) = self.tileset().get_coordinates(&pos);

                // First paint the floor tiles
                if level.is_square(&pos) {
                    self.paint_tile(canvas, Tile::Square, x, y);
                } else {
                    self.paint_tile(canvas, Tile::Floor, x, y);
                }

                // Add the shadows
                let flags = get_shadow_flags(&level, &pos);
                for f in &[
                    ShadowFlags::N_EDGE,
                    ShadowFlags::S_EDGE,
                    ShadowFlags::E_EDGE,
                    ShadowFlags::W_EDGE,
                    ShadowFlags::NE_CORNER,
                    ShadowFlags::NW_CORNER,
                    ShadowFlags::SE_CORNER,
                    ShadowFlags::SW_CORNER,
                ] {
                    if flags.contains(*f) {
                        self.paint_tile(canvas, Tile::Shadow(*f), x, y);
                    }
                }

                // Draw the other items
                let z = y - self.tileset().offset();
                if level.is_wall(&pos) {
                    self.paint_tile(canvas, Tile::Wall, x, z);
                }
                if level.is_box(&pos) {
                    self.paint_tile(canvas, Tile::Rock, x, z);
                }
                if level.is_player(&pos) {
                    self.paint_tile(canvas, Tile::Player, x, z);
                }
            }
        }
    }

    /// Paints the status bar
    fn paint_status_bar(&mut self, canvas: &mut Canvas<Window>, level: &Level) {
        let prev_color = canvas.draw_color();
        canvas.set_draw_color(self.bar_color);
        let rect = Rect::new(
            0,
            (self.screen_size.1 - self.bar_height) as i32,
            self.screen_size.0,
            self.bar_height,
        );
        canvas.fill_rect(rect).unwrap();
        canvas.set_draw_color(prev_color);

        // Paints the number of moves
        let s = format!("# moves: {}", level.get_steps());
        self.paint_status_text(canvas, &s, StatusBarLocation::FlushLeft);

        // Paints the level's title
        self.paint_status_text(canvas, level.title(), StatusBarLocation::FlushRight);
    }

    /// Paints text in the status bar
    fn paint_status_text(
        &mut self,
        canvas: &mut Canvas<Window>,
        text: &str,
        location: StatusBarLocation,
    ) {
        let surface = self.font.render(text).blended(self.bar_text_color).unwrap();
        let creator = canvas.texture_creator();
        let texture = creator.create_texture_from_surface(&surface).unwrap();
        let margin = 4;
        let (w, h) = {
            let q = texture.query();
            (q.width, q.height)
        };
        let (x, y) = match location {
            StatusBarLocation::FlushLeft => {
                (margin as i32, (self.screen_size.1 - margin - h) as i32)
            }
            StatusBarLocation::FlushRight => (
                (self.screen_size.0 - margin - w) as i32,
                (self.screen_size.1 - margin - h) as i32,
            ),
        };
        canvas
            .copy(&texture, None, Some(Rect::new(x, y, w, h)))
            .unwrap();
    }

    /// Paints a tile at the given coordinates.
    fn paint_tile(&mut self, canvas: &mut Canvas<Window>, tile: Tile, x: i32, y: i32) {
        let (col, row) = self.tileset().location(tile).unwrap_or_else(|| {
            panic!("No image for this tile: {:?}", tile);
        });
        let tile_rect = self.tileset().get_tile_rect(col, row);
        let target_rect = Some(Rect::new(
            x,
            y,
            self.tileset().width(),
            self.tileset().height(),
        ));
        canvas
            .copy(self.tileset().texture(), tile_rect, target_rect)
            .unwrap();
    }

    /// Returns the size of the drawing scaled to fit onto the screen.
    fn get_scaled_rendering_size(&self, level: &Level) -> (u32, u32) {
        let render_size = self.tileset().get_rendering_size(level.extents());
        let width_ratio = f64::from(self.screen_size.0) / f64::from(render_size.0);
        let h = self.screen_size.1 - self.bar_height;
        let height_ratio = f64::from(h) / f64::from(render_size.1);
        let ratio = f64::min(1.0, f64::min(width_ratio, height_ratio));

        let scale = |sz: u32| (ratio * f64::from(sz)).floor() as u32;

        (scale(render_size.0), scale(render_size.1))
    }

    /// Returns the Rect of an image of given dimensions so that it's centered on the screen.
    fn get_centered_image_rect(&self, img_size: (u32, u32)) -> Option<Rect> {
        let x = (self.screen_size.0 - img_size.0) as i32 / 2;
        let y = (self.screen_size.1 - self.bar_height - img_size.1) as i32 / 2;
        Some(Rect::new(x, y, img_size.0, img_size.1))
    }

    fn tileset(&self) -> &Tileset {
        self.selector.select()
    }
}

/// Returns the shadow flags for a particular position in the given level.
fn get_shadow_flags(level: &Level, pos: &Position) -> ShadowFlags {
    let north = pos.neighbor(Direction::Up);
    let south = pos.neighbor(Direction::Down);
    let west = pos.neighbor(Direction::Left);
    let east = pos.neighbor(Direction::Right);

    let mut flags = ShadowFlags::empty();
    if level.is_wall(&north) {
        flags |= ShadowFlags::N_EDGE;
    }
    if level.is_wall(&south) {
        flags |= ShadowFlags::S_EDGE;
    }
    if level.is_wall(&west) {
        flags |= ShadowFlags::W_EDGE;
    }
    if level.is_wall(&east) {
        flags |= ShadowFlags::E_EDGE;
    }
    if level.is_wall(&north.neighbor(Direction::Right))
        && !flags.intersects(ShadowFlags::N_EDGE | ShadowFlags::E_EDGE)
    {
        flags |= ShadowFlags::NE_CORNER;
    }
    if level.is_wall(&north.neighbor(Direction::Left))
        && !flags.intersects(ShadowFlags::N_EDGE | ShadowFlags::W_EDGE)
    {
        flags |= ShadowFlags::NW_CORNER;
    }
    if level.is_wall(&south.neighbor(Direction::Right))
        && !flags.intersects(ShadowFlags::S_EDGE | ShadowFlags::E_EDGE)
    {
        flags |= ShadowFlags::SE_CORNER;
    }
    if level.is_wall(&south.neighbor(Direction::Left))
        && !flags.intersects(ShadowFlags::S_EDGE | ShadowFlags::W_EDGE)
    {
        flags |= ShadowFlags::SW_CORNER;
    }
    flags
}
