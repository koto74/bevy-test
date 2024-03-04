use bevy::{
    math::bounding::{Aabb2d, BoundingCircle, IntersectsVolume},
    prelude::*,
    sprite::MaterialMesh2dBundle,
};

const WALL_THICKNESS: f32 = 10.0;
// x coordinates
const LEFT_WALL: f32 = -450.;
const RIGHT_WALL: f32 = 450.;
// y coordinates
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

const BALL_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, -50.0, 1.0);
const BALL_DIAMETER: f32 = 30.0;
const BALL_SPEED: f32 = 400.0;
const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins) // ウィンドウやスケジュールなどの基本的な機能を追加
        .add_event::<CollisionEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                apply_velocity,
                check_for_collisions,
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

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Collider;

#[derive(Event, Default)]
struct CollisionEvent;

#[derive(Bundle)]
struct WallBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
        }
    }

    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;
        // 定数を間違えていないか確認します
        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            }
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}

impl WallBundle {
    fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    // Vec2をVec3に変換する必要があります。これは、スプライトの順序を決定するために使用されます
                    translation: location.position().extend(0.0),
                    // 2Dオブジェクトのzスケールは常に1.0でなければなりません。
                    // そうでないと、スプライトの順序が予想外の方法で影響を受けます。
                    // 詳細は https://github.com/bevyengine/bevy/issues/4149 を参照してください
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Ball
    commands.spawn((
        // 2Dメッシュとマテリアルを束ねたもの
        // 2Dメッシュは円形のメッシュを作成
        // マテリアルはボールの色を指定
        // トランスフォームはボールの位置と大きさを指定
        // その他のフィールドはデフォルト値
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::default()).into(),
            material: materials.add(BALL_COLOR),
            transform: Transform::from_translation(BALL_STARTING_POSITION)
                // ボールの大きさを指定
                .with_scale(Vec2::splat(BALL_DIAMETER).extend(1.0)),
            ..default()
        },
        Ball,
        Velocity(INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED),
    ));

    // Walls
    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Top));

}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.0.x * time.delta_seconds();
        transform.translation.y += velocity.0.y * time.delta_seconds();
    }
}

fn check_for_collisions(
    mut commands: Commands,
    mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<(Entity, &Transform), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let (mut ball_velocity, ball_transform) = ball_query.single_mut();

    // すべてのコライダーとボールの位置をチェック
    for (collider_entity, transform) in &collider_query {
        let collision = collide_with_side(
            BoundingCircle::new(ball_transform.translation.truncate(), BALL_DIAMETER / 2.),
            Aabb2d::new(
                transform.translation.truncate(),
                transform.scale.truncate() / 2.,
            ),
        );

        if let Some(collision) = collision {
            // 他のシステムが衝突に反応できるように、衝突イベントを送信します
            collision_events.send_default();

            // 衝突したときにボールを反射させる
            let mut reflect_x = false;
            let mut reflect_y = false;

            // only reflect if the ball's velocity is going in the opposite direction of the
            // collision
            match collision {
                Collision::Left => reflect_x = ball_velocity.0.x > 0.0,
                Collision::Right => reflect_x = ball_velocity.0.x < 0.0,
                Collision::Top => reflect_y = ball_velocity.0.y < 0.0,
                Collision::Bottom => reflect_y = ball_velocity.0.y > 0.0,
            }

            // x軸で何かに当たった場合は、速度をx軸に反射させる
            if reflect_x {
                ball_velocity.0.x = -ball_velocity.0.x;
            }

            // y軸で何かに当たった場合は、速度をy軸に反射させる
            if reflect_y {
                ball_velocity.0.y = -ball_velocity.0.y;
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

fn collide_with_side(ball: BoundingCircle, wall: Aabb2d) -> Option<Collision> {
    // ボールが壁と交差していない場合は、Noneを返します
    if !ball.intersects(&wall) {
        return None;
    }

    let closest = wall.closest_point(ball.center);
    let offset = ball.center - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}
