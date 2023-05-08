use cw_storage_plus::Item;

use abstract_core::objects::fee::UsageFee;

pub const SWAP_FEE: Item<UsageFee> = Item::new("swap_fee");
