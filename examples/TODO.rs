//! abc

trait InputModifier {}
trait InputCondition {}

struct ContextInstance;
impl ContextInstance {
    fn bind<A>(&mut self) -> &mut BindAction {
        todo!()
    }

    fn bind_to<A>(mut self, bind: impl IntoInputBind) -> Self {
        self.bind::<A>().to(bind);
        self
    }
}

struct WasdKeys;
enum GamepadStick {
    Left,
    Right,
}

struct DeadZone;
struct Scale;

//

struct Move;

struct Look;

fn main() {
    let mut ctx = ContextInstance;
    ctx.bind::<Move>()
        .to(KeyCode::KeyA)
        .to(KeyCode::KeyD.with_modifier(DeadZone))
        .with_modifier(Scale);

    /*
    BindConfigSet {
        binds: [
            One(BindConfig {
                input: KeyCode::KeyA,
                modifiers: [],
                conditions: [],
            }),
            One(BindConfig {
                input: KeyCode::KeyD,
                modifiers: [DeadZone],
                conditions: [],
            })
        ],
        modifiers: [Scale],
        conditions: [],
    }
    */

    ctx.bind_to::<Move>(
        (KeyCode::KeyA, KeyCode::KeyD.with_modifier(DeadZone)).with_modifier(Scale),
    );

    /*
    BindConfigSet {
        binds: [
            Set(BindConfigSet {
                binds: [
                    One(BindConfig {
                        input: KeyCode::KeyA,
                        modifiers: [],
                        conditions: [],
                    }),
                    One(BindConfig {
                        input: KeyCode::KeyD,
                        modifiers: [DeadZone],
                        conditions: [],
                    }),
                ],
                modifiers: [Scale],
                conditions: [],
            }),
        ],
        modifiers: [],
        conditions: [],
    }
    */

    /*
    BindConfigs::Many [
        BindConfigs::One(
            BindConfig {
                input: Input::Keyboard {
                    key: KeyCode::A,
                    modifiers: []
                },
                modifiers: [],
                conditions: []
            }
        ),
        BindConfigs::One(
            BindConfig {
                input: Input::Keyboard {
                    key: KeyCode::D,
                    modifiers: []
                },
                modifiers: [Box(DeadZone)],
                conditions: []
            },
        )
    ]

    */

    ctx.bind::<Look>();
}
