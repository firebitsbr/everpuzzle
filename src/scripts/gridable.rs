// NOTE(Skytrias): maybe put this into mod.rs?
// set of functions both garbage and the block share
pub trait Gridable {
    fn is_idle(self) -> bool;
    fn is_hang(self) -> bool;
    fn is_clear(self) -> bool;

    fn clear_started(self) -> bool;
    fn hang_started(self) -> bool;

    fn hang_finished(self) -> bool;
    fn clear_finished(self) -> bool;

    fn to_hang(&mut self, counter: u32);
    fn to_clear(&mut self);
}
