# CosmWasm Staking

An Abstract-API contract that handles staking and unbonding interactions with staking providers. 

## Naming Convention

In order to easily identify and relate contracts to on-chain addresses we follow the following conventions:

*Staking AddressEntry*: a `ContractEntry` that is formatted as {provider}:stake/{staking_asset_entry}
*Staking AssetEntry*: a `AssetEntry` of the token that is stakeable
