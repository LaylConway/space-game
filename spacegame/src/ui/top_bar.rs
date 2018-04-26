use {
    rivr::{
        attributes::{PanelSize, AxisSize, PanelBox, Orientation, Srgba},
        panels::{ButtonPanel, StackPanel, LabelPanel, EmptyPanel},
        Ui, Event, PanelId, FontId,
    },

    spacegame_game::{
        state::{BuildState, BuildChoice},
        ObjectClasses, ObjectClassId,
    },
};

pub struct TopBar {
    buid_menu: BuildMenu,
    game_menu: GameMenu,
}

impl TopBar {
    pub fn new(ui: &mut Ui, font: FontId, object_classes: &ObjectClasses) -> (Self, PanelId) {
        let (buid_menu, buid_menu_id) = BuildMenu::new(ui, font, object_classes);
        let (game_menu, game_menu_id) = GameMenu::new(ui, font);

        let spacer = EmptyPanel::new(
            PanelSize::new(AxisSize::Max, AxisSize::Max),
            PanelBox::default()
        );
        let spacer_id = ui.add_panel(spacer);

        let mut top_bar = StackPanel::new(
            PanelSize::new(AxisSize::Max, AxisSize::Min),
            PanelBox {
                background: Some(Srgba::new(1.0, 1.0, 1.0, 0.8)),
                .. PanelBox::default()
            },
            Orientation::Horizontal, 0.0,
        );
        top_bar.add_child(buid_menu_id);
        top_bar.add_child(spacer_id);
        top_bar.add_child(game_menu_id);
        let top_bar_id = ui.add_panel(top_bar);

        (TopBar {
            buid_menu,
            game_menu,
        }, top_bar_id)
    }

    pub fn update(&self, build_state: &mut BuildState) {
        self.buid_menu.update(build_state);
        self.game_menu.update();
    }
}

struct BuildMenu {
    build_floor_pressed: Event,
    destroy_pressed: Event,
    destroy_all_pressed: Event,

    build_buttons: Vec<(Event, ObjectClassId)>,
}

impl BuildMenu {
    pub fn new(ui: &mut Ui, font: FontId, object_classes: &ObjectClasses) -> (Self, PanelId) {
        let (build_floor_button_id, build_floor_pressed) =
            labeled_button(ui, "Build Floor", font);
        let (destroy_button_id, destroy_pressed) =
            labeled_button(ui, "Destroy", font);
        let (destroy_all_button_id, destroy_all_pressed) =
            labeled_button(ui, "Destroy All", font);

        let mut build_menu = StackPanel::new(
            PanelSize::new(AxisSize::Min, AxisSize::Min),
            PanelBox::default(),
            Orientation::Horizontal, 3.0,
        );
        build_menu.add_child(build_floor_button_id);

        // Add all the buttons for different objects
        let mut build_buttons = Vec::new();
        for (id, class) in object_classes.classes().iter().enumerate() {
            let (build_button_id, build_pressed) =
                labeled_button(ui, &format!("Build {}", class.friendly_name), font);
            build_menu.add_child(build_button_id);
            build_buttons.push((build_pressed, ObjectClassId { id }));
        }

        build_menu.add_child(destroy_button_id);
        build_menu.add_child(destroy_all_button_id);
        let build_menu_id = ui.add_panel(build_menu);

        (BuildMenu {
            build_floor_pressed,
            destroy_pressed,
            destroy_all_pressed,

            build_buttons,
        }, build_menu_id)
    }

    pub fn update(&self, build_state: &mut BuildState) {
        if self.build_floor_pressed.check() {
            build_state.choice = BuildChoice::Floor;
        }
        if self.destroy_pressed.check() {
            build_state.choice = BuildChoice::Destroy;
        }
        if self.destroy_all_pressed.check() {
            build_state.choice = BuildChoice::DestroyAll;
        }

        for (event, id) in &self.build_buttons {
            if event.check() {
                build_state.choice = BuildChoice::Object(*id);
            }
        }
    }
}

struct GameMenu {
    save_pressed: Event,
}

impl GameMenu {
    pub fn new(ui: &mut Ui, font: FontId) -> (Self, PanelId) {
        let (save_button_id, save_pressed) =
            labeled_button(ui, "Save", font);

        let mut game_menu = StackPanel::new(
            PanelSize::new(AxisSize::Min, AxisSize::Min),
            PanelBox::default(),
            Orientation::Horizontal, 3.0,
        );
        game_menu.add_child(save_button_id);
        let game_menu_id = ui.add_panel(game_menu);

        (GameMenu {
            save_pressed,
        }, game_menu_id)
    }

    pub fn update(&self) {
    }
}

fn labeled_button(
    ui: &mut Ui, text: &str, font: FontId
) -> (PanelId, Event) {
    let label = LabelPanel::new(ui, text, font, 12.0).unwrap();
    let label_id = ui.add_panel(label);

    let button = ButtonPanel::new(
        PanelSize::absolute(84.0, 24.0),
        PanelBox {
            background: Some(Srgba::new(1.0, 1.0, 1.0, 1.0)),
            background_hovering: Some(Srgba::new(0.95, 0.95, 0.95, 1.0)),
            border_radius: 3.0,
            .. PanelBox::default()
        },
        Some(label_id), 3.0,
    );
    let pressed = button.event_pressed();
    let button_id = ui.add_panel(button);

    (button_id, pressed)
}
