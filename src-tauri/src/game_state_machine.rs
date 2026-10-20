use std::time::{SystemTime, UNIX_EPOCH};

use crate::bag_manager::{BagManager, SlotDelta};
use crate::models::{AppSettings, GameEvent, ParsedEvent};
use crate::pricing::PricingManager;

const MAP_COST_PROTO_NAMES: &[&str] = &["Spv3Open", "ClimbTowerOpen", "MapActivate"];

const TRADE_PROTO_NAMES: &[&str] = &[
    "XchgReceive",
    "ExchangeItem",
    "XchgRecall",
    "XchgForSale",
    "Push2",
    "UnequipSkill",
    "AuctionBuyout",
    "BoothBuy",
];

const SORT_PROTO_NAMES: &[&str] = &["ResetItemsLayout", "SortBag", "InventorySort"];

const TRADE_UI_PAGES: &[&str] = &[
    "TradeHouse",
    "Auction",
    "Booth",
    "Mail",
    "Xchg",
    "Spv3",
    "Exchange",
];

const HUB_SCENE_PATTERNS: &[&str] = &[
    "YuJinZhiXiBiNanSuo",
    "UIMainLevel",
    "LoginScene",
    "SelectRole",
    "ZhuCheng",
    "ShengTingZhuangYuan",
    "XZ_",
    "DD_ZaWuJieQu",
    "lobby",
    "hub",
    "town",
    "hideout",
];

const EXPENSE_ITEM_KEYWORDS: &[&str] = &[
    "Compass",
    "Resonance",
    "Beacon",
    "Map",
    "Key",
    "Card",
    "Edict",
    "Invitation",
    "Probe",
    "Scalpel",
    "Astrolabe",
    "Luo Pan",
    "Luo Zhi",
    "Bei Ke",
];

const EXPENSE_BUFFER_TIMEOUT_SECS: u64 = 60;

#[derive(Clone, PartialEq, Debug)]
pub enum GameState {
    Unknown,
    Hub,
    InMap { level_type: u32, level_id: u32 },
}

#[derive(Clone, PartialEq, Debug)]
pub enum EventContext {
    Normal,
    Pickup,
    MapOpen,
    TradeHouse,
    Sorting,
}

#[derive(Clone, Debug)]
struct PendingExpense {
    item_id: String,
    item_name: String,
    qty: u32,
    price: f64,
    timestamp: u64,
}

pub struct GameStateMachine {
    pub state: GameState,
    pub context: EventContext,

    pub bag: BagManager,
    pub settings: AppSettings,

    pending_expenses: Vec<PendingExpense>,
    last_map_level_id: Option<u32>,
    last_proto_name: Option<String>,
}

impl GameStateMachine {
    pub fn new(settings: AppSettings) -> Self {
        Self {
            state: GameState::Unknown,
            context: EventContext::Normal,
            bag: BagManager::new(),
            settings,
            pending_expenses: Vec::new(),
            last_map_level_id: None,
            last_proto_name: None,
        }
    }

    pub fn update_settings(&mut self, settings: AppSettings) {
        self.settings = settings;
    }

    pub fn process(&mut self, event: ParsedEvent, pricing: &PricingManager) -> Vec<GameEvent> {
        match event {
            ParsedEvent::LevelChange {
                level_type,
                level_id,
                ..
            } => self.handle_level_change(level_type, level_id),
            ParsedEvent::SceneChange { scene_name } => self.handle_scene_change(&scene_name),
            ParsedEvent::ProtoStart { proto_name } => {
                self.handle_proto_start(&proto_name);
                vec![]
            }
            ParsedEvent::ProtoEnd { proto_name } => self.handle_proto_end(&proto_name),
            ParsedEvent::UiPageChange { page_name, .. } => {
                self.handle_ui_page(&page_name);
                vec![]
            }
            ParsedEvent::BagInit {
                page_id,
                slot_id,
                item_id,
                qty,
            } => {
                self.bag.init_slot(page_id, slot_id, &item_id, qty);
                vec![]
            }
            ParsedEvent::BagModify {
                page_id,
                slot_id,
                item_id,
                qty,
            } => self.handle_bag_modify(page_id, slot_id, &item_id, qty, pricing),
            ParsedEvent::BagRemove { page_id, slot_id } => {
                self.handle_bag_remove(page_id, slot_id, pricing)
            }
            ParsedEvent::ItemAdd {
                item_id,
                page_id,
                slot_id,
                qty,
            } => self.handle_item_add(&item_id, page_id, slot_id, qty, pricing),
        }
    }

    fn handle_level_change(&mut self, level_type: u32, level_id: u32) -> Vec<GameEvent> {
        let is_map = level_type >= 2 && !Self::is_hub_level_id(level_id);
        let mut results = vec![GameEvent::ZoneChange {
            level_type,
            level_id,
            is_hub: !is_map,
        }];

        if is_map {
            if self.last_map_level_id != Some(level_id) {
                self.last_map_level_id = Some(level_id);
                self.state = GameState::InMap {
                    level_type,
                    level_id,
                };
                self.context = EventContext::Normal;
                self.last_proto_name = None;

                results.push(GameEvent::MapStart {
                    level_type,
                    level_id,
                });
                let flushed = self.flush_expenses();
                results.extend(flushed);
            }
        } else {
            self.state = GameState::Hub;
            self.last_map_level_id = None;
        }

        results
    }

    fn handle_scene_change(&mut self, scene_name: &str) -> Vec<GameEvent> {
        if Self::is_hub_scene(scene_name) && !matches!(self.state, GameState::InMap { .. }) {
            self.state = GameState::Hub;
            self.last_map_level_id = None;
        }
        vec![]
    }

    fn handle_proto_start(&mut self, proto_name: &str) {
        self.last_proto_name = Some(proto_name.to_string());
        if SORT_PROTO_NAMES.contains(&proto_name) {
            self.context = EventContext::Sorting;
            self.bag.reset();
        } else if MAP_COST_PROTO_NAMES.contains(&proto_name) {
            self.context = EventContext::MapOpen;
        } else if TRADE_PROTO_NAMES.contains(&proto_name) {
            self.context = EventContext::TradeHouse;
        } else if proto_name == "PickItems" {
            self.context = EventContext::Pickup;
        } else {
            self.context = EventContext::Normal;
        }
    }

    fn handle_proto_end(&mut self, proto_name: &str) -> Vec<GameEvent> {
        let is_sort = SORT_PROTO_NAMES.contains(&proto_name);
        match self.context {
            EventContext::MapOpen => {}
            EventContext::TradeHouse if TRADE_PROTO_NAMES.contains(&proto_name) => {}
            _ => {
                self.context = EventContext::Normal;
            }
        }
        if is_sort {
            vec![GameEvent::SortComplete]
        } else {
            vec![]
        }
    }

    fn handle_ui_page(&mut self, page_name: &str) {
        let is_trade = TRADE_UI_PAGES.iter().any(|kw| page_name.contains(kw));
        if is_trade {
            self.context = EventContext::TradeHouse;
        } else if self.context == EventContext::TradeHouse {
            self.context = EventContext::Normal;
        }
    }

    fn handle_bag_modify(
        &mut self,
        page_id: u32,
        slot_id: u32,
        item_id: &str,
        qty: u32,
        pricing: &PricingManager,
    ) -> Vec<GameEvent> {
        if self.context == EventContext::Sorting {
            return vec![];
        }

        let delta = self.bag.update_slot(page_id, slot_id, item_id, qty);
        self.classify_delta(delta, item_id, page_id, pricing)
    }

    fn handle_bag_remove(
        &mut self,
        page_id: u32,
        slot_id: u32,
        pricing: &PricingManager,
    ) -> Vec<GameEvent> {
        if self.context == EventContext::Sorting {
            return vec![];
        }

        if let Some(old) = self.bag.remove_slot(page_id, slot_id) {
            let delta = SlotDelta::Lost(old.item_id.clone(), old.qty);
            return self.classify_delta(delta, &old.item_id, page_id, pricing);
        }
        vec![]
    }

    fn handle_item_add(
        &mut self,
        item_id: &str,
        page_id: i32,
        slot_id: i32,
        qty: u32,
        pricing: &PricingManager,
    ) -> Vec<GameEvent> {
        if self.context == EventContext::Sorting {
            return vec![];
        }

        let in_main_inv = page_id >= 100 && page_id <= 104;
        if !in_main_inv {
            return vec![];
        }

        if page_id >= 0 && slot_id >= 0 {
            self.bag
                .init_slot(page_id as u32, slot_id as u32, item_id, qty);
        }

        let is_loot_context = matches!(self.context, EventContext::Pickup | EventContext::Normal);
        let is_trade = self.context == EventContext::TradeHouse;

        if is_trade {
            if self.settings.ignore_auction {
                return vec![];
            }
            let price = pricing.get_price_sync(item_id);
            let item_name = crate::db::DbManager::get_item_name(item_id);
            return vec![GameEvent::LootDrop {
                item_id: item_id.to_string(),
                item_name,
                qty,
                price,
                valuation: pricing.get_quote(item_id),
                season_id: pricing.season_id(),
            }];
        }

        if matches!(self.state, GameState::InMap { .. }) && is_loot_context {
            let price = pricing.get_price_sync(item_id);
            let item_name = crate::db::DbManager::get_item_name(item_id);
            return vec![GameEvent::LootDrop {
                item_id: item_id.to_string(),
                item_name,
                qty,
                price,
                valuation: pricing.get_quote(item_id),
                season_id: pricing.season_id(),
            }];
        }
        vec![]
    }

    fn classify_delta(
        &mut self,
        delta: SlotDelta,
        _item_id: &str,
        page_id: u32,
        pricing: &PricingManager,
    ) -> Vec<GameEvent> {
        match delta {
            SlotDelta::Gained(id, qty)
            | SlotDelta::Swapped {
                new_item_id: id,
                new_qty: qty,
            } => self.handle_gain(&id, qty, page_id, pricing),
            SlotDelta::Lost(id, qty) => self.handle_loss(&id, qty, pricing),
            SlotDelta::None => vec![],
        }
    }

    fn handle_gain(
        &mut self,
        item_id: &str,
        qty: u32,
        page_id: u32,
        pricing: &PricingManager,
    ) -> Vec<GameEvent> {
        if !BagManager::is_main_inventory(page_id) {
            return vec![];
        }

        match self.context {
            EventContext::TradeHouse => {
                if self.settings.ignore_auction {
                    return vec![];
                }
                let price = pricing.get_price_sync(item_id);
                let item_name = crate::db::DbManager::get_item_name(item_id);
                return vec![GameEvent::LootDrop {
                    item_id: item_id.to_string(),
                    item_name,
                    qty,
                    price,
                    valuation: pricing.get_quote(item_id),
                    season_id: pricing.season_id(),
                }];
            }
            EventContext::Pickup => {
                if !matches!(self.state, GameState::InMap { .. }) {
                    return vec![];
                }
                let price = pricing.get_price_sync(item_id);
                let item_name = crate::db::DbManager::get_item_name(item_id);
                return vec![GameEvent::LootDrop {
                    item_id: item_id.to_string(),
                    item_name,
                    qty,
                    price,
                    valuation: pricing.get_quote(item_id),
                    season_id: pricing.season_id(),
                }];
            }
            _ => vec![],
        }
    }

    fn handle_loss(&mut self, item_id: &str, qty: u32, pricing: &PricingManager) -> Vec<GameEvent> {
        let item_name = crate::db::DbManager::get_item_name(item_id);
        let price = pricing.get_price_sync(item_id);
        let is_consumable = Self::is_expense_item(&item_name);

        match self.context {
            EventContext::MapOpen => {
                self.buffer_expense(item_id, &item_name, qty, price);
                vec![]
            }
            EventContext::TradeHouse => {
                if self.settings.ignore_auction {
                    return vec![];
                }
                vec![GameEvent::Expense {
                    item_id: item_id.to_string(),
                    item_name: item_name.clone(),
                    qty,
                    total: price * qty as f64,
                }]
            }
            EventContext::Normal => {
                if is_consumable {
                    self.buffer_expense(item_id, &item_name, qty, price);
                }
                vec![]
            }
            EventContext::Sorting | EventContext::Pickup => vec![],
        }
    }

    fn buffer_expense(&mut self, item_id: &str, item_name: &str, qty: u32, price: f64) {
        let now = now_secs();
        self.expire_expenses(now);
        self.pending_expenses.push(PendingExpense {
            item_id: item_id.to_string(),
            item_name: item_name.to_string(),
            qty,
            price,
            timestamp: now,
        });
    }

    fn flush_expenses(&mut self) -> Vec<GameEvent> {
        let now = now_secs();
        self.expire_expenses(now);
        self.pending_expenses
            .drain(..)
            .map(|e| GameEvent::Expense {
                item_id: e.item_id,
                item_name: e.item_name,
                qty: e.qty,
                total: e.price * e.qty as f64,
            })
            .collect()
    }

    fn expire_expenses(&mut self, now: u64) {
        self.pending_expenses
            .retain(|e| now.saturating_sub(e.timestamp) < EXPENSE_BUFFER_TIMEOUT_SECS);
    }

    fn is_hub_scene(scene_name: &str) -> bool {
        let lower = scene_name.to_lowercase();
        HUB_SCENE_PATTERNS
            .iter()
            .any(|pat| lower.contains(&pat.to_lowercase()))
            && !scene_name.contains("ShengTingZhuangYuan000")
    }

    fn is_hub_level_id(level_id: u32) -> bool {
        matches!(level_id, 0 | 1)
    }

    fn is_expense_item(item_name: &str) -> bool {
        let lower = item_name.to_lowercase();
        EXPENSE_ITEM_KEYWORDS
            .iter()
            .any(|kw| lower.contains(&kw.to_lowercase()))
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
