use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Debug)]
#[serde(tag = "type")]
pub enum GameEvent {
    MapStart {
        level_type: u32,
        level_id: u32,
    },
    ZoneChange {
        level_type: u32,
        level_id: u32,
        is_hub: bool,
    },
    LootDrop {
        item_id: String,
        item_name: String,
        qty: u32,
        price: f64,
        valuation: crate::pricing::PriceQuote,
        season_id: u32,
    },
    Expense {
        item_id: String,
        item_name: String,
        qty: u32,
        total: f64,
    },
    LoadingStart {
        total_lines: u64,
    },
    LoadingDone,
    InventoryRefreshed {
        value: f64,
        items: Vec<InventoryItem>,
    },
    SortComplete,
}

#[derive(Clone, Serialize, Debug)]
pub struct InventoryItem {
    pub item_id: String,
    pub item_name: String,
    pub qty: u32,
    pub price: f64,
    pub valuation: crate::pricing::PriceQuote,
    pub season_id: u32,
}

#[derive(Clone, Debug)]
pub enum ParsedEvent {
    LevelChange {
        level_uid: u32,
        level_type: u32,
        level_id: u32,
    },
    SceneChange {
        scene_name: String,
    },
    BagInit {
        page_id: u32,
        slot_id: u32,
        item_id: String,
        qty: u32,
    },
    BagModify {
        page_id: u32,
        slot_id: u32,
        item_id: String,
        qty: u32,
    },
    BagRemove {
        page_id: u32,
        slot_id: u32,
    },
    ItemAdd {
        item_id: String,
        page_id: i32,
        slot_id: i32,
        qty: u32,
    },
    ProtoStart {
        proto_name: String,
    },
    ProtoEnd {
        proto_name: String,
    },
    UiPageChange {
        page_name: String,
        state: String,
    },
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct AppSettings {
    #[serde(default)]
    pub ignore_auction: bool,
    #[serde(default)]
    pub apply_tax: bool,
}
