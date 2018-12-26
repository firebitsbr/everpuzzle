use crate::{
    block_states::{Clear, Hang, Land},
    components::Block,
};

// changes the current block's state to a new one
pub fn change_state(b: &mut Block, new_state: &'static str) {
    if b.state == new_state {
        return;
    }

    // call the current block's state's exit function
    match b.state {
        "LAND" => Land::exit(b),
        "CLEAR" => Clear::exit(b),
        _ => (),
    }

    b.state = new_state;

    // call the current block's state's enter function
    match b.state {
        "HANG" => Hang::enter(b),
        "LAND" => Land::enter(b),
        "CLEAR" => Clear::enter(b),
        _ => (),
    }
}
