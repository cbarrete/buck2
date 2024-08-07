/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

//! Dice operations for legacy configuration

use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;

use allocative::Allocative;
use async_trait::async_trait;
use buck2_core::cells::name::CellName;
use buck2_futures::cancellation::CancellationContext;
use derive_more::Display;
use dice::DiceComputations;
use dice::DiceProjectionComputations;
use dice::DiceTransactionUpdater;
use dice::InjectedKey;
use dice::Key;
use dice::OpaqueValue;
use dice::ProjectionKey;
use dupe::Dupe;

use crate::dice::cells::HasCellResolver;
use crate::legacy_configs::cells::BuckConfigBasedCells;
use crate::legacy_configs::cells::ExternalBuckconfigData;
use crate::legacy_configs::configs::LegacyBuckConfig;
use crate::legacy_configs::configs::LegacyBuckConfigs;
use crate::legacy_configs::key::BuckconfigKeyRef;
use crate::legacy_configs::view::LegacyBuckConfigView;

/// Buckconfig view which queries buckconfig entry from DICE.
#[derive(Clone, Dupe)]
pub struct OpaqueLegacyBuckConfigOnDice {
    config: Arc<OpaqueValue<LegacyBuckConfigForCellKey>>,
}

impl std::fmt::Debug for OpaqueLegacyBuckConfigOnDice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LegacyBuckConfigOnDice")
            .field("config", &self.config)
            .finish()
    }
}

impl OpaqueLegacyBuckConfigOnDice {
    pub fn lookup(
        &self,
        ctx: &mut DiceComputations,
        key: BuckconfigKeyRef,
    ) -> anyhow::Result<Option<Arc<str>>> {
        let BuckconfigKeyRef { section, property } = key;
        Ok(ctx.projection(
            &*self.config,
            &LegacyBuckConfigPropertyProjectionKey {
                section: section.to_owned(),
                property: property.to_owned(),
            },
        )?)
    }

    pub fn view<'a, 'd>(
        &'a self,
        ctx: &'a mut DiceComputations<'d>,
    ) -> LegacyBuckConfigOnDice<'a, 'd> {
        LegacyBuckConfigOnDice { ctx, config: self }
    }
}

pub struct LegacyBuckConfigOnDice<'a, 'd> {
    ctx: &'a mut DiceComputations<'d>,
    config: &'a OpaqueLegacyBuckConfigOnDice,
}

impl LegacyBuckConfigOnDice<'_, '_> {
    pub fn parse<T: FromStr>(&mut self, key: BuckconfigKeyRef) -> anyhow::Result<Option<T>>
    where
        anyhow::Error: From<<T as FromStr>::Err>,
    {
        LegacyBuckConfig::parse_value(key, self.get(key)?.as_deref())
    }
}

impl std::fmt::Debug for LegacyBuckConfigOnDice<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LegacyBuckConfigOnDice")
            .field("config", &self.config)
            .finish()
    }
}

impl<'a, 'd> LegacyBuckConfigView for LegacyBuckConfigOnDice<'a, 'd> {
    fn get(&mut self, key: BuckconfigKeyRef) -> anyhow::Result<Option<Arc<str>>> {
        self.config.lookup(self.ctx, key)
    }
}

pub trait HasInjectedLegacyConfigs {
    /// Use this function carefully: a computation which fetches this key will be recomputed
    /// if any buckconfig property changes.
    ///
    /// Consider using `get_legacy_config_property` instead.
    fn get_injected_legacy_configs(
        &mut self,
    ) -> impl Future<Output = anyhow::Result<LegacyBuckConfigs>>;

    /// Checks if LegacyBuckConfigsKey has been set in the DICE graph.
    fn is_injected_legacy_configs_key_set(&mut self) -> impl Future<Output = anyhow::Result<bool>>;

    fn get_injected_external_buckconfig_data(
        &mut self,
    ) -> impl Future<Output = anyhow::Result<Arc<ExternalBuckconfigData>>>;

    fn is_injected_external_buckconfig_data_key_set(
        &mut self,
    ) -> impl Future<Output = anyhow::Result<bool>>;
}

#[async_trait]
pub trait HasLegacyConfigs {
    /// Get buckconfigs.
    ///
    /// This operation does not record buckconfig as a dependency of current computation.
    /// Accessing specific buckconfig property, records that key as dependency.
    async fn get_legacy_config_on_dice(
        &mut self,
        cell_name: CellName,
    ) -> anyhow::Result<OpaqueLegacyBuckConfigOnDice>;

    async fn get_legacy_root_config_on_dice(
        &mut self,
    ) -> anyhow::Result<OpaqueLegacyBuckConfigOnDice>;

    /// Use this function carefully: a computation which fetches this key will be recomputed
    /// if any buckconfig property changes.
    ///
    /// Consider using `get_legacy_config_property` instead.
    async fn get_legacy_config_for_cell(
        &mut self,
        cell_name: CellName,
    ) -> buck2_error::Result<LegacyBuckConfig>;

    async fn get_legacy_config_property(
        &mut self,
        cell_name: CellName,
        key: BuckconfigKeyRef<'_>,
    ) -> anyhow::Result<Option<Arc<str>>>;

    async fn parse_legacy_config_property<T: FromStr>(
        &mut self,
        cell_name: CellName,
        key: BuckconfigKeyRef<'_>,
    ) -> anyhow::Result<Option<T>>
    where
        anyhow::Error: From<<T as FromStr>::Err>,
        T: Send + Sync + 'static;
}

pub trait SetLegacyConfigs {
    fn set_legacy_configs(&mut self, legacy_configs: LegacyBuckConfigs) -> anyhow::Result<()>;

    fn set_none_legacy_configs(&mut self) -> anyhow::Result<()>;

    fn set_legacy_config_external_data(
        &mut self,
        overrides: Arc<ExternalBuckconfigData>,
    ) -> anyhow::Result<()>;

    fn set_none_legacy_config_external_data(&mut self) -> anyhow::Result<()>;
}

#[derive(Clone, Dupe, Display, Debug, Eq, Hash, PartialEq, Allocative)]
#[display(fmt = "{:?}", self)]
struct LegacyBuckConfigKey;

impl InjectedKey for LegacyBuckConfigKey {
    type Value = Option<LegacyBuckConfigs>;

    fn equality(x: &Self::Value, y: &Self::Value) -> bool {
        match (x, y) {
            (Some(x), Some(y)) => x.compare(y),
            (None, None) => true,
            (_, _) => false,
        }
    }
}

#[derive(Clone, Dupe, Display, Debug, Eq, Hash, PartialEq, Allocative)]
#[display(fmt = "{:?}", self)]
struct LegacyExternalBuckConfigDataKey;

impl InjectedKey for LegacyExternalBuckConfigDataKey {
    type Value = Option<Arc<ExternalBuckconfigData>>;

    fn equality(x: &Self::Value, y: &Self::Value) -> bool {
        x == y
    }
}

#[derive(Clone, Display, Debug, Hash, Eq, PartialEq, Allocative)]
#[display(fmt = "LegacyBuckConfigForCellKey({})", "self.cell_name")]
struct LegacyBuckConfigForCellKey {
    cell_name: CellName,
}

#[async_trait]
impl Key for LegacyBuckConfigForCellKey {
    type Value = buck2_error::Result<LegacyBuckConfig>;

    async fn compute(
        &self,
        ctx: &mut DiceComputations,
        _cancellations: &CancellationContext,
    ) -> buck2_error::Result<LegacyBuckConfig> {
        let cells = ctx.get_cell_resolver().await?;
        let this_cell = cells.get(self.cell_name)?;
        if this_cell.external().is_some() {
            return BuckConfigBasedCells::parse_single_cell_with_dice(ctx, this_cell.path())
                .await
                .map_err(Into::into);
        }

        let legacy_configs = ctx.get_injected_legacy_configs().await?;
        legacy_configs
            .get(self.cell_name)
            .map(|x| x.dupe())
            .map_err(buck2_error::Error::from)
    }

    fn equality(x: &Self::Value, y: &Self::Value) -> bool {
        match (x, y) {
            (Ok(x), Ok(y)) => x.compare(y),
            _ => false,
        }
    }
}

/// The computation `LegacyBuckConfigForCellKey` computation might encounter an error.
///
/// We can't return that error immediately, because we only compute the opaque value. We could
/// return the error when doing the projection to the buckconfig values, but that would result in us
/// increasing the size of the value returned from that computation. Instead, we'll use a different
/// projection key to extract just the error from the cell computation, and compute that when
/// constructing the `OpaqueLegacyBuckConfigOnDice`.
#[derive(Debug, Display, Hash, Eq, PartialEq, Clone, Allocative)]
struct LegacyBuckConfigErrorKey();

impl ProjectionKey for LegacyBuckConfigErrorKey {
    type DeriveFromKey = LegacyBuckConfigForCellKey;
    type Value = Option<buck2_error::Error>;

    fn compute(
        &self,
        config: &buck2_error::Result<LegacyBuckConfig>,
        _ctx: &DiceProjectionComputations,
    ) -> Option<buck2_error::Error> {
        config.as_ref().err().cloned()
    }

    fn equality(x: &Self::Value, y: &Self::Value) -> bool {
        x.is_none() && y.is_none()
    }
}

#[derive(Debug, Display, Hash, Eq, PartialEq, Clone, Allocative)]
#[display(fmt = "{}.{}", section, property)]
struct LegacyBuckConfigPropertyProjectionKey {
    section: String,
    property: String,
}

impl ProjectionKey for LegacyBuckConfigPropertyProjectionKey {
    type DeriveFromKey = LegacyBuckConfigForCellKey;
    type Value = Option<Arc<str>>;

    fn compute(
        &self,
        config: &buck2_error::Result<LegacyBuckConfig>,
        _ctx: &DiceProjectionComputations,
    ) -> Option<Arc<str>> {
        // See the comment in `LegacyBuckConfigErrorKey` for why this is safe
        let config = config.as_ref().unwrap();
        config
            .get(BuckconfigKeyRef {
                section: &self.section,
                property: &self.property,
            })
            .map(|s| s.to_owned().into())
    }

    fn equality(x: &Self::Value, y: &Self::Value) -> bool {
        x == y
    }
}

impl HasInjectedLegacyConfigs for DiceComputations<'_> {
    async fn get_injected_legacy_configs(&mut self) -> anyhow::Result<LegacyBuckConfigs> {
        self.compute(&LegacyBuckConfigKey).await?.ok_or_else(|| {
            panic!("Tried to retrieve LegacyBuckConfigKey from the graph, but key has None value")
        })
    }

    async fn is_injected_legacy_configs_key_set(&mut self) -> anyhow::Result<bool> {
        Ok(self.compute(&LegacyBuckConfigKey).await?.is_some())
    }

    async fn get_injected_external_buckconfig_data(
        &mut self,
    ) -> anyhow::Result<Arc<ExternalBuckconfigData>> {
        self.compute(&LegacyExternalBuckConfigDataKey).await?.ok_or_else(|| {
            panic!("Tried to retrieve LegacyBuckConfigOverridesKey from the graph, but key has None value")
        })
    }

    async fn is_injected_external_buckconfig_data_key_set(&mut self) -> anyhow::Result<bool> {
        Ok(self
            .compute(&LegacyExternalBuckConfigDataKey)
            .await?
            .is_some())
    }
}

#[async_trait]
impl HasLegacyConfigs for DiceComputations<'_> {
    async fn get_legacy_config_on_dice(
        &mut self,
        cell_name: CellName,
    ) -> anyhow::Result<OpaqueLegacyBuckConfigOnDice> {
        let config = self
            .compute_opaque(&LegacyBuckConfigForCellKey { cell_name })
            .await?;
        if let Some(error) = self.projection(&config, &LegacyBuckConfigErrorKey())? {
            return Err(error.into());
        }
        Ok(OpaqueLegacyBuckConfigOnDice {
            config: Arc::new(config),
        })
    }

    async fn get_legacy_root_config_on_dice(
        &mut self,
    ) -> anyhow::Result<OpaqueLegacyBuckConfigOnDice> {
        let cell_resolver = self.get_cell_resolver().await?;
        self.get_legacy_config_on_dice(cell_resolver.root_cell())
            .await
    }

    async fn get_legacy_config_for_cell(
        &mut self,
        cell_name: CellName,
    ) -> buck2_error::Result<LegacyBuckConfig> {
        self.compute(&LegacyBuckConfigForCellKey { cell_name })
            .await?
    }

    async fn get_legacy_config_property(
        &mut self,
        cell_name: CellName,
        key: BuckconfigKeyRef<'_>,
    ) -> anyhow::Result<Option<Arc<str>>> {
        self.get_legacy_config_on_dice(cell_name)
            .await?
            .lookup(self, key)
    }

    async fn parse_legacy_config_property<T: FromStr>(
        &mut self,
        cell_name: CellName,
        key: BuckconfigKeyRef<'_>,
    ) -> anyhow::Result<Option<T>>
    where
        anyhow::Error: From<<T as FromStr>::Err>,
        T: Send + Sync + 'static,
    {
        LegacyBuckConfig::parse_value(
            key,
            self.get_legacy_config_property(cell_name, key)
                .await?
                .as_deref(),
        )
    }
}

impl SetLegacyConfigs for DiceTransactionUpdater {
    fn set_legacy_configs(&mut self, legacy_configs: LegacyBuckConfigs) -> anyhow::Result<()> {
        Ok(self.changed_to(vec![(LegacyBuckConfigKey, Some(legacy_configs))])?)
    }

    fn set_none_legacy_configs(&mut self) -> anyhow::Result<()> {
        Ok(self.changed_to(vec![(LegacyBuckConfigKey, None)])?)
    }

    fn set_legacy_config_external_data(
        &mut self,
        data: Arc<ExternalBuckconfigData>,
    ) -> anyhow::Result<()> {
        Ok(self.changed_to(vec![(LegacyExternalBuckConfigDataKey, Some(data))])?)
    }

    fn set_none_legacy_config_external_data(&mut self) -> anyhow::Result<()> {
        Ok(self.changed_to(vec![(LegacyExternalBuckConfigDataKey, None)])?)
    }
}

#[cfg(test)]
mod tests {
    use buck2_cli_proto::ConfigOverride;
    use buck2_core::cells::name::CellName;
    use dice::InjectedKey;

    use crate::legacy_configs::configs::testing::parse_with_config_args;
    use crate::legacy_configs::configs::LegacyBuckConfigs;
    use crate::legacy_configs::dice::LegacyBuckConfigKey;

    #[test]
    fn config_equals() -> anyhow::Result<()> {
        let path = "test";
        let config1 = Some(LegacyBuckConfigs::new(hashmap![
            CellName::testing_new("cell1")
            => {
                parse_with_config_args(
                    &[("test", "[sec1]\na=b\n[sec2]\nx=y")],
                    path,
                    &[ConfigOverride::flag("sec1.a=c")],
                )?
            },
            CellName::testing_new("cell2")
            => {
                parse_with_config_args(
                    &[("test", "[sec1]\nx=y\n[sec2]\na=b")],
                    path,
                    &[],
                )?
            }
        ]));

        let config2 = Some(LegacyBuckConfigs::new(hashmap![
            CellName::testing_new("cell1")
            => {
                parse_with_config_args(
                    &[("test", "[sec1]\na=b\n[sec2]\nx=y")],
                    path,
                    &[ConfigOverride::flag("sec1.a=c")],
                )?
            },
        ]));

        let config3 = Some(LegacyBuckConfigs::new(hashmap![
            CellName::testing_new("cell1")
            => {
                parse_with_config_args(
                    &[("test", "[sec1]\na=c\n[sec2]\nx=y")],
                    path,
                    &[],
                )?
            },
        ]));

        let config4 = Some(LegacyBuckConfigs::new(hashmap![
            CellName::testing_new("cell1")
            => {
                parse_with_config_args(
                    &[("test", "[sec1]\na=b\n[sec2]\nx=y")],
                    path,
                    &[ConfigOverride::flag("sec1.d=e")],
                )?
            },
        ]));

        let config5: Option<LegacyBuckConfigs> = None;
        let config6: Option<LegacyBuckConfigs> = None;

        assert_eq!(LegacyBuckConfigKey::equality(&config1, &config1), true);
        assert_eq!(LegacyBuckConfigKey::equality(&config2, &config2), true);
        assert_eq!(LegacyBuckConfigKey::equality(&config3, &config3), true);
        assert_eq!(LegacyBuckConfigKey::equality(&config4, &config4), true);
        assert_eq!(LegacyBuckConfigKey::equality(&config1, &config2), false);
        assert_eq!(LegacyBuckConfigKey::equality(&config1, &config3), false);
        assert_eq!(LegacyBuckConfigKey::equality(&config1, &config4), false);
        assert_eq!(LegacyBuckConfigKey::equality(&config2, &config3), true);
        assert_eq!(LegacyBuckConfigKey::equality(&config2, &config4), false);
        assert_eq!(LegacyBuckConfigKey::equality(&config3, &config4), false);
        assert_eq!(LegacyBuckConfigKey::equality(&config5, &config1), false);
        assert_eq!(LegacyBuckConfigKey::equality(&config5, &config6), true);

        Ok(())
    }
}
