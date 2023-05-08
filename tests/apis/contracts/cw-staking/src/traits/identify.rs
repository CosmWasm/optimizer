/// Identify a staking provider by its name and ibc status
pub trait Identify {
    fn name(&self) -> &'static str;
}
