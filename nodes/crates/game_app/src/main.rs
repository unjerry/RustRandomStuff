use nodes_core::GameCore;
use nodes_render::Renderer;
use nodes_ui::UiState;

fn main() {
    let mut game = GameCore::new();
    let mut renderer = Renderer::new();
    let mut ui = UiState::new();

    game.update();
    ui.update(&game);
    renderer.draw_frame(&game);

    println!("nodes scaffold ran one tick: {}", game.tick());
}

