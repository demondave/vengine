use super::Game;

pub trait Scene {
    fn on_load(&mut self, _game: &mut Game) {}

    fn render(&mut self, game: &mut Game);

    fn on_unload(&mut self, _game: &mut Game) {}

    fn on_current(&mut self, _game: &mut Game) {}
}
