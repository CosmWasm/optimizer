use crate::error::StakingError;
use crate::CwStakingAdapter;
use cosmwasm_std::{StdError, StdResult};

#[cfg(feature = "terra")]
use super::astroport::{Astroport, ASTROPORT};
#[cfg(any(feature = "juno", feature = "osmosis"))]
pub use crate::providers::osmosis::{Osmosis, OSMOSIS};
#[cfg(feature = "juno")]
pub use crate::providers::{
    junoswap::{JunoSwap, JUNOSWAP},
    wyndex::{WynDex, WYNDEX},
};

pub(crate) fn is_over_ibc(provider: &str) -> StdResult<bool> {
    match provider {
        #[cfg(feature = "juno")]
        JUNOSWAP => Ok(false),
        #[cfg(feature = "juno")]
        WYNDEX => Ok(false),
        #[cfg(feature = "terra")]
        ASTROPORT => Ok(false),
        #[cfg(feature = "juno")]
        OSMOSIS => Ok(true),
        _ => Err(StdError::generic_err(format!(
            "Unknown provider {provider}"
        ))),
    }
}

/// Given the provider name, return the local provider implementation
pub(crate) fn resolve_local_provider(
    name: &str,
) -> Result<Box<dyn CwStakingAdapter>, StakingError> {
    match name {
        #[cfg(feature = "juno")]
        JUNOSWAP => Ok(Box::<JunoSwap>::default()),
        #[cfg(feature = "juno")]
        WYNDEX => Ok(Box::<WynDex>::default()),
        #[cfg(feature = "osmosis")]
        OSMOSIS => Ok(Box::new(Osmosis::default())),
        #[cfg(feature = "terra")]
        ASTROPORT => Ok(Box::<Astroport>::default()),
        _ => Err(StakingError::ForeignDex(name.to_owned())),
    }
}
