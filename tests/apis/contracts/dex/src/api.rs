// TODO: this should be moved to the public dex package
// It cannot be in abstract-os because it does not have a dependency on sdk (as it shouldn't)
use crate::{
    msg::{
        AskAsset, DexAction, DexApiExecuteMsg, DexExecuteMsg, DexName, DexQueryMsg, OfferAsset,
        SimulateSwapResponse, SwapRouter,
    },
    EXCHANGE,
};
use abstract_core::objects::{module::ModuleId, AssetEntry};
use abstract_sdk::ApiInterface;
use abstract_sdk::{
    features::{AccountIdentification, Dependencies},
    AbstractSdkResult,
};
use cosmwasm_std::{CosmosMsg, Decimal, Deps, Uint128};
use serde::de::DeserializeOwned;

/// Interact with the dex api in your module.
pub trait DexInterface: AccountIdentification + Dependencies {
    /// Construct a new dex interface
    /// Params:
    /// - deps: the deps object
    /// - dex_name: the name of the dex to interact with
    fn dex<'a>(&'a self, deps: Deps<'a>, dex_name: DexName) -> Dex<Self> {
        Dex {
            base: self,
            deps,
            dex_name,
            dex_module_id: EXCHANGE,
        }
    }
}

impl<T: AccountIdentification + Dependencies> DexInterface for T {}

#[derive(Clone)]
pub struct Dex<'a, T: DexInterface> {
    base: &'a T,
    dex_name: DexName,
    dex_module_id: ModuleId<'a>,
    deps: Deps<'a>,
}

impl<'a, T: DexInterface> Dex<'a, T> {
    /// Set the module id for the
    pub fn with_module_id(self, module_id: ModuleId<'a>) -> Self {
        Self {
            dex_module_id: module_id,
            ..self
        }
    }
    fn dex_name(&self) -> DexName {
        self.dex_name.clone()
    }
    fn dex_module_id(&self) -> ModuleId {
        self.dex_module_id
    }
    fn request(&self, action: DexAction) -> AbstractSdkResult<CosmosMsg> {
        let modules = self.base.apis(self.deps);

        modules.request(
            self.dex_module_id(),
            DexApiExecuteMsg::from(DexExecuteMsg {
                dex: self.dex_name(),
                action,
            }),
        )
    }

    pub fn swap(
        &self,
        offer_asset: OfferAsset,
        ask_asset: AssetEntry,
        max_spread: Option<Decimal>,
        belief_price: Option<Decimal>,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.request(DexAction::Swap {
            offer_asset,
            ask_asset,
            belief_price,
            max_spread,
        })
    }

    pub fn custom_swap(
        &self,
        offer_assets: Vec<OfferAsset>,
        ask_assets: Vec<AskAsset>,
        max_spread: Option<Decimal>,
        router: Option<SwapRouter>,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.request(DexAction::CustomSwap {
            offer_assets,
            ask_assets,
            max_spread,
            router,
        })
    }

    pub fn provide_liquidity(
        &self,
        assets: Vec<OfferAsset>,
        max_spread: Option<Decimal>,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.request(DexAction::ProvideLiquidity { assets, max_spread })
    }

    pub fn provide_liquidity_symmetric(
        &self,
        offer_asset: OfferAsset,
        paired_assets: Vec<AssetEntry>,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.request(DexAction::ProvideLiquiditySymmetric {
            offer_asset,
            paired_assets,
        })
    }

    pub fn withdraw_liquidity(
        &self,
        lp_token: AssetEntry,
        amount: Uint128,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.request(DexAction::WithdrawLiquidity { lp_token, amount })
    }
}

impl<'a, T: DexInterface> Dex<'a, T> {
    fn query<R: DeserializeOwned>(&self, query_msg: DexQueryMsg) -> AbstractSdkResult<R> {
        let modules = self.base.apis(self.deps);
        modules.query(EXCHANGE, query_msg)
    }
    pub fn simulate_swap(
        &self,
        offer_asset: OfferAsset,
        ask_asset: AssetEntry,
    ) -> AbstractSdkResult<SimulateSwapResponse> {
        let response: SimulateSwapResponse = self.query(DexQueryMsg::SimulateSwap {
            dex: Some(self.dex_name()),
            offer_asset,
            ask_asset,
        })?;
        Ok(response)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::msg::ExecuteMsg;
    use abstract_core::api::ApiRequestMsg;
    use abstract_sdk::mock_module::MockModule;
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::wasm_execute;
    use speculoos::prelude::*;

    fn expected_request_with_test_proxy(request: DexExecuteMsg) -> ExecuteMsg {
        ApiRequestMsg {
            proxy_address: Some(abstract_testing::prelude::TEST_PROXY.to_string()),
            request: request.into(),
        }
        .into()
    }

    #[test]
    fn swap_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let stub = MockModule::new();
        let dex = stub
            .dex(deps.as_ref(), "junoswap".into())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let dex_name = "junoswap".to_string();
        let offer_asset = OfferAsset::new("juno", 1000u128);
        let ask_asset = AssetEntry::new("uusd");
        let max_spread = Some(Decimal::percent(1));
        let belief_price = Some(Decimal::percent(2));

        let expected = expected_request_with_test_proxy(DexExecuteMsg {
            dex: dex_name,
            action: DexAction::Swap {
                offer_asset: offer_asset.clone(),
                ask_asset: ask_asset.clone(),
                max_spread,
                belief_price,
            },
        });

        let actual = dex.swap(offer_asset, ask_asset, max_spread, belief_price);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(
            abstract_testing::prelude::TEST_MODULE_ADDRESS,
            &expected,
            vec![],
        )
        .unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn custom_swap_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let stub = MockModule::new();
        let dex_name = "astroport".to_string();

        let dex = stub
            .dex(deps.as_ref(), dex_name.clone())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let offer_assets = vec![OfferAsset::new("juno", 1000u128)];
        let ask_assets = vec![AskAsset::new("uusd", 1000u128)];
        let max_spread = Some(Decimal::percent(1));
        let router = Some(SwapRouter::Custom("custom_router".to_string()));

        let expected = expected_request_with_test_proxy(DexExecuteMsg {
            dex: dex_name,
            action: DexAction::CustomSwap {
                offer_assets: offer_assets.clone(),
                ask_assets: ask_assets.clone(),
                max_spread,
                router: router.clone(),
            },
        });

        let actual = dex.custom_swap(offer_assets, ask_assets, max_spread, router);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(
            abstract_testing::prelude::TEST_MODULE_ADDRESS,
            &expected,
            vec![],
        )
        .unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn provide_liquidity_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let stub = MockModule::new();
        let dex_name = "junoswap".to_string();

        let dex = stub
            .dex(deps.as_ref(), dex_name.clone())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let assets = vec![OfferAsset::new("taco", 1000u128)];
        let max_spread = Some(Decimal::percent(1));

        let expected = expected_request_with_test_proxy(DexExecuteMsg {
            dex: dex_name,
            action: DexAction::ProvideLiquidity {
                assets: assets.clone(),
                max_spread,
            },
        });

        let actual = dex.provide_liquidity(assets, max_spread);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(
            abstract_testing::prelude::TEST_MODULE_ADDRESS,
            &expected,
            vec![],
        )
        .unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn provide_liquidity_symmetric_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let stub = MockModule::new();
        let dex_name = "junoswap".to_string();

        let dex = stub
            .dex(deps.as_ref(), dex_name.clone())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let offer = OfferAsset::new("taco", 1000u128);
        let paired = vec![AssetEntry::new("bell")];
        let _max_spread = Some(Decimal::percent(1));

        let expected = expected_request_with_test_proxy(DexExecuteMsg {
            dex: dex_name,
            action: DexAction::ProvideLiquiditySymmetric {
                offer_asset: offer.clone(),
                paired_assets: paired.clone(),
            },
        });

        let actual = dex.provide_liquidity_symmetric(offer, paired);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(
            abstract_testing::prelude::TEST_MODULE_ADDRESS,
            &expected,
            vec![],
        )
        .unwrap();

        assert_that!(actual).is_equal_to(expected);
    }

    #[test]
    fn withdraw_liquidity_msg() {
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let stub = MockModule::new();
        let dex_name = "junoswap".to_string();

        let dex = stub
            .dex(deps.as_ref(), dex_name.clone())
            .with_module_id(abstract_testing::prelude::TEST_MODULE_ID);

        let lp_token = AssetEntry::new("taco");
        let withdraw_amount: Uint128 = 1000u128.into();

        let expected = expected_request_with_test_proxy(DexExecuteMsg {
            dex: dex_name,
            action: DexAction::WithdrawLiquidity {
                lp_token: lp_token.clone(),
                amount: withdraw_amount,
            },
        });

        let actual = dex.withdraw_liquidity(lp_token, withdraw_amount);

        assert_that!(actual).is_ok();

        let actual = match actual.unwrap() {
            CosmosMsg::Wasm(msg) => msg,
            _ => panic!("expected wasm msg"),
        };
        let expected = wasm_execute(
            abstract_testing::prelude::TEST_MODULE_ADDRESS,
            &expected,
            vec![],
        )
        .unwrap();

        assert_that!(actual).is_equal_to(expected);
    }
}
