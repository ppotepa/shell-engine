use engine_core::scene::BehaviorSpec;
use crate::{
    Behavior, BlinkBehavior, BobBehavior, FollowBehavior, MenuCarouselBehavior,
    MenuCarouselObjectBehavior, MenuSelectedBehavior, RhaiScriptBehavior, SelectedArrowsBehavior,
    StageVisibilityBehavior, TimedVisibilityBehavior,
};

/// Resolves a `BehaviorSpec` into a boxed `Behavior` runtime instance.
///
/// The default `BuiltInBehaviorFactory` handles all engine-defined behavior names.
/// Mods or tests can provide an alternative or chained factory without touching
/// the engine's dispatch chain.
pub trait BehaviorFactory: Send + Sync {
    fn create(&self, spec: &BehaviorSpec) -> Option<Box<dyn Behavior + Send + Sync>>;
}

/// Resolves all engine built-in behavior names.
/// This is the single authoritative dispatch point for built-in behaviors — replaces
/// the inline if/else chain that was previously in `behavior::built_in_behavior`.
pub struct BuiltInBehaviorFactory;

impl BehaviorFactory for BuiltInBehaviorFactory {
    fn create(&self, spec: &BehaviorSpec) -> Option<Box<dyn Behavior + Send + Sync>> {
        let name = spec.name.trim();
        if name.eq_ignore_ascii_case("blink") {
            Some(Box::new(BlinkBehavior::from_params(&spec.params)))
        } else if name.eq_ignore_ascii_case("bob") {
            Some(Box::new(BobBehavior::from_params(&spec.params)))
        } else if name.eq_ignore_ascii_case("follow") {
            Some(Box::new(FollowBehavior::from_params(&spec.params)))
        } else if name.eq_ignore_ascii_case("menu-carousel") {
            Some(Box::new(MenuCarouselBehavior::from_params(&spec.params)))
        } else if name.eq_ignore_ascii_case("menu-carousel-object") {
            Some(Box::new(MenuCarouselObjectBehavior::from_params(&spec.params)))
        } else if name.eq_ignore_ascii_case("rhai-script") {
            Some(Box::new(RhaiScriptBehavior::from_params(&spec.params)))
        } else if name.eq_ignore_ascii_case("menu-selected") {
            Some(Box::new(MenuSelectedBehavior::from_params(&spec.params)))
        } else if name.eq_ignore_ascii_case("selected-arrows") {
            Some(Box::new(SelectedArrowsBehavior::from_params(&spec.params)))
        } else if name.eq_ignore_ascii_case("stage-visibility") {
            Some(Box::new(StageVisibilityBehavior::from_params(&spec.params)))
        } else if name.eq_ignore_ascii_case("timed-visibility") {
            Some(Box::new(TimedVisibilityBehavior::from_params(&spec.params)))
        } else {
            None
        }
    }
}
