#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ClawCondition, ClawAction, ClawTarget};
    use crate::utils::{fmt_price, fmt_volume};

    #[test]
    fn test_claw_above_triggers() {
        let mut t = ClawTarget::new("BTCUSDT", ClawCondition::Above, 70000.0, ClawAction::Alert);
        assert!(!t.check(69999.0));
        assert!(t.check(70000.0));
        assert!(t.check(70001.0));
    }

    #[test]
    fn test_claw_below_triggers() {
        let t = ClawTarget::new("ETHUSDT", ClawCondition::Below, 3000.0, ClawAction::Alert);
        assert!(!t.check(3001.0));
        assert!(t.check(3000.0));
        assert!(t.check(2999.0));
    }

    #[test]
    fn test_claw_percent_triggers() {
        let t = ClawTarget::new("SOLUSDT", ClawCondition::PercentChange(5.0), 100.0, ClawAction::Alert);
        // 5% of 100 = 5 → trigger at >= 105 or <= 95
        assert!(!t.check(102.0)); // only 2% change
        assert!(t.check(105.0));  // exactly 5%
        assert!(t.check(95.0));   // exactly 5% down
        assert!(t.check(110.0));  // 10% — also triggers
    }

    #[test]
    fn test_claw_does_not_double_trigger() {
        let mut t = ClawTarget::new("BTCUSDT", ClawCondition::Above, 70000.0, ClawAction::Alert);
        assert!(t.check(70001.0));
        t.triggered = true;
        assert!(!t.check(70001.0)); // already triggered
    }

    #[test]
    fn test_fmt_price_small() {
        let s = fmt_price(0.00000123);
        assert!(s.len() > 5);
    }

    #[test]
    fn test_fmt_price_large() {
        let s = fmt_price(67432.10);
        assert!(s.starts_with("67432"));
    }

    #[test]
    fn test_fmt_volume_millions() {
        let s = fmt_volume("1500000");
        assert!(s.ends_with('M'));
    }

    #[test]
    fn test_fmt_volume_thousands() {
        let s = fmt_volume("25000");
        assert!(s.ends_with('K'));
    }

    #[test]
    fn test_skill_manifest_has_commands() {
        let manifest = crate::skill::skill_manifest();
        assert!(manifest["commands"].is_array());
        let cmds = manifest["commands"].as_array().unwrap();
        assert!(cmds.len() >= 7);
    }

    #[test]
    fn test_skill_result_ok() {
        let r = crate::types::SkillResult::ok(serde_json::json!({"price": 67000.0}));
        assert_eq!(r.skill, "binance-claw");
        assert!(matches!(r.status, crate::types::SkillStatus::Ok));
    }

    #[test]
    fn test_skill_result_error() {
        let r = crate::types::SkillResult::error("something went wrong");
        assert!(matches!(r.status, crate::types::SkillStatus::Error));
        assert_eq!(r.data["error"], "something went wrong");
    }
}
