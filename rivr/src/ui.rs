use {
    metrohash::{MetroHashMap},

    panels::{Panel},
};

pub struct Ui {
    panels: MetroHashMap<u32, Box<Panel>>,
    next_id: u32,
}

impl Ui {
    pub fn new() -> Self {
        Ui {
            panels: Default::default(),
            next_id: 0,
        }
    }

    pub fn add_panel<P: Panel>(&mut self, panel: P) -> PanelId {
        let id = self.next_id;
        self.next_id += 1;

        self.panels.insert(id, Box::new(panel));

        PanelId { id }
    }
}

#[derive(Copy, Clone)]
pub struct PanelId {
    id: u32,
}
