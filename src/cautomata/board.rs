use sdl2::render::{Canvas, RenderTarget};

pub trait Board {
    fn new(rows: usize, cols: usize) -> Self;
    fn next_gen(&mut self);
    fn randomize(&mut self);
    fn clear(&mut self);
    fn draw<T: RenderTarget>(&self, c: &mut Canvas<T>, width: u32, height: u32);
}
