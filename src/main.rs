use macroquad::prelude::*;

#[macroquad::main("Flecsirogue")]
async fn main() {
    // TODO look into CameraWrapper in my other macroquad project
    let scale = 4.0;
    let camera = Camera2D {
        zoom: vec2(scale / screen_width(), scale / screen_height()),
        rotation: 0.,
        offset: vec2(0., 0.0),
        target: vec2(screen_width()/scale, screen_height()/scale),
        render_target: None,
        viewport: None,
    };

    set_camera(&camera);

    let rogues_texture = load_texture("assets/32rogues/rogues.png").await.unwrap();
    rogues_texture.set_filter(FilterMode::Nearest);

    let mut pos = vec2(32., 32.);

    loop {
        clear_background(BLACK);

	if(is_key_pressed(KeyCode::W)) {
	    pos.y -= 32.0;
	}
	if(is_key_pressed(KeyCode::S)) {
	    pos.y += 32.0;
	}
	if(is_key_pressed(KeyCode::A)) {
	    pos.x -= 32.0;
	}
	if(is_key_pressed(KeyCode::D)) {
	    pos.x += 32.0;
	}

        //draw_texture(&tileset, 50., 50., WHITE);
        draw_texture_ex(
            &rogues_texture,
            pos.x,
            pos.y,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(0., 0., 32., 32.)),
                ..Default::default()
            },
        );

	let text = format!("{:?}", camera.screen_to_world(mouse_position().into()));
        draw_text(&text, 20.0, 20.0, 30.0, WHITE);

        next_frame().await
    }
}
