//! Actual parser -> FSM -> serialized Tauri payload contracts (no GUI needed).
#[cfg(test)]
mod tests {
    use crate::{
        game_state_machine::GameStateMachine,
        models::{AppSettings, GameEvent},
        parser::LogParser,
        pricing::{PriceStatus, PricingManager},
    };
    #[test]
    fn pickup_unknown_price_keeps_quantity_and_serializes_explicit_unknown() {
        let pricing = PricingManager::new(
            1601,
            std::env::temp_dir().join("deanfinite-integration-unused"),
        );
        let mut parser = LogParser::new();
        let mut fsm = GameStateMachine::new(AppSettings::default());
        for line in [
            "LevelMgr@ LevelUid, LevelType, LevelId = 1 3 123",
            "ItemChange@ ProtoName=PickItems start",
        ] {
            fsm.process(parser.parse_line(line).unwrap(), &pricing);
        }
        let event = parser
            .parse_line("ItemChange@ Add Id=123456 BagNum=3 PageId=100 SlotId=1")
            .unwrap();
        let events = fsm.process(event, &pricing);
        assert_eq!(events.len(), 1);
        match &events[0] {
            GameEvent::LootDrop {
                qty,
                valuation,
                season_id,
                ..
            } => {
                assert_eq!(*qty, 3);
                assert_eq!(*season_id, 1601);
                assert_eq!(valuation.status, PriceStatus::Unknown);
                assert!(valuation.value.is_none());
            }
            _ => panic!("Not a drop"),
        };
        let json = serde_json::to_value(&events[0]).unwrap();
        assert_eq!(json["type"], "LootDrop");
        assert!(json["valuation"]["value"].is_null());
        // The follow-up slot modification to the same quantity is not a second drop.
        let event = parser
            .parse_line(
                "BagMgr@:Modfy BagItem PageId = 100 SlotId = 1 ConfigBaseId = 123456 Num = 3",
            )
            .unwrap();
        assert!(fsm.process(event, &pricing).is_empty());
    }
    #[test]
    fn auction_exclusion_still_applies_and_price_search_has_no_side_effect() {
        let pricing = PricingManager::new(
            1601,
            std::env::temp_dir().join("deanfinite-integration-unused"),
        );
        let mut parser = LogParser::new();
        let mut fsm = GameStateMachine::new(AppSettings {
            ignore_auction: true,
            apply_tax: false,
        });
        for line in [
            "ItemChange@ ProtoName=XchgReceive start",
            "ItemChange@ Add Id=123456 BagNum=2 PageId=100 SlotId=1",
        ] {
            assert!(fsm
                .process(parser.parse_line(line).unwrap(), &pricing)
                .is_empty());
        }
        for line in ["XchgSearchPrice +refer [123456]", "+unitPrices+1 [99]"] {
            assert!(parser.parse_line(line).is_none());
        }
        assert!(pricing.get_quote("123456").value.is_none());
    }
}
