use ggez::{Context, GameResult};
use slog::{Logger};
use serde::{Deserialize, Serialize};
use rmp_serde::{Deserializer, Serializer};

use markedly::template::{Template};
use markedly::{Ui, ComponentEvents, UiContext};

use model::ship::{Ship};

pub struct SaveInputController {
    events: ComponentEvents,
}

impl SaveInputController {
    pub fn new(
        ctx: &mut Context, ui: &mut Ui, ui_context: &UiContext,
    ) -> GameResult<Self> {
        let template_file = ctx.filesystem.open("/markedly/save-input.mark")?;
        let template = Template::from_reader(template_file)?;
        let events = ui.insert_template(&template, None, "top-menu", ui_context)
            .map_err(|e| format!("{:#?}", e))?;

        Ok(SaveInputController {
            events,
        })
    }

    pub fn update(
        &mut self, log: &Logger, ctx: &mut Context, ship: &mut Ship
    ) -> GameResult<()> {
        while let Some(event) = self.events.next() {
            match event.as_str() {
                "load-game" => {
                    info!(log, "Loading game");

                    let mut file = ctx.filesystem.open("/save.mp")?;
                    let mut de = Deserializer::new(&mut file);
                    *ship = Deserialize::deserialize(&mut de).unwrap();
                },
                "save-game" => {
                    info!(log, "Saving game");

                    let mut file = ctx.filesystem.create("/save.mp")?;
                    ship.serialize(&mut Serializer::new(&mut file)).unwrap();
                },
                "new-game" => {
                    info!(log, "Creating new game");

                    *ship = Ship::starter(log);
                },
                _ => {},
            }
        }

        Ok(())
    }
}
