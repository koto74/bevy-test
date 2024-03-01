use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins) // ウィンドウやスケジュールなどの基本的な機能を追加
        .add_systems(
            Startup,
            (
                spawn_camera2d, // 2Dカメラを追加
                spawn_text2d,   // 2Dテキストを追加
            )
        )
        .add_systems(
            Update,
            (
                bevy::window::close_on_esc, // Escキーでウィンドウを閉じる
            )
        )
        .run();
}

fn spawn_camera2d(mut cmds: Commands) {
    cmds.spawn(Camera2dBundle::default());
}

fn spawn_text2d(mut cmds: Commands) {
    let textstyle = TextStyle { font_size: 100.0, ..default() };
    let text = Text::from_section("Hello, world!", textstyle);
    cmds.spawn(Text2dBundle { text, ..default() });
}
