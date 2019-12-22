use matches::matches;
use serde::Deserialize;
use serde::de::{Deserializer, Error as _};

use crate::config::{Config, BrokerConfig};
use crate::core::GenericResult;
use crate::currency::{Cash, CashAssets};
use crate::types::{Decimal, TradeType};

use self::commissions::{
    CommissionSpec as OldCommissionSpec,
    CommissionSpecBuilder as OldCommissionSpecBuilder,
};

mod commissions;

#[derive(Debug, Clone, Copy)]
pub enum Broker {
    Bcs,
    InteractiveBrokers,
    OpenBroker,
}

impl Broker {
    pub fn get_info(self, config: &Config) -> GenericResult<BrokerInfo> {
        Ok(BrokerInfo {
            name: self.get_name(),
            config: self.get_config(config)?,
            commission_spec: self.get_commission_spec(),
            allow_sparse_broker_statements: matches!(self, Broker::Bcs),
        })
    }

    fn get_name(self) -> &'static str {
        match self {
            Broker::Bcs => "ООО «Компания БКС»",
            Broker::InteractiveBrokers => "Interactive Brokers LLC",
            Broker::OpenBroker => "АО «Открытие Брокер»",
        }
    }

    fn get_config(self, config: &Config) -> GenericResult<BrokerConfig> {
        Ok(config.brokers.as_ref().and_then(|brokers| {
            match self {
                Broker::Bcs => brokers.bcs.as_ref(),
                Broker::InteractiveBrokers => brokers.interactive_brokers.as_ref(),
                Broker::OpenBroker => brokers.open_broker.as_ref(),
            }
        }).ok_or_else(|| format!(
            "{} configuration is not set in the configuration file", self.get_name())
        )?.clone())
    }

    fn get_commission_spec(self) -> OldCommissionSpec {
        match self {
            // BCS has tiered commissions that aren't supported yet, so use some average now
            Broker::Bcs => OldCommissionSpecBuilder::new()
                .percent(dec!(0.057))
                .build().unwrap(),

            Broker::InteractiveBrokers => OldCommissionSpecBuilder::new()
                .minimum(Cash::new("USD", dec!(1)))
                .per_share(Cash::new("USD", dec!(0.005)))
                .maximum_percent(dec!(1))

                // Stock selling fee
                .transaction_fee(TradeType::Sell, OldCommissionSpecBuilder::new()
                    .percent(dec!(0.0013))
                    .build().unwrap())

                // FINRA trading activity fee
                .transaction_fee(TradeType::Sell, OldCommissionSpecBuilder::new()
                    .per_share(Cash::new("USD", dec!(0.000119)))
                    .build().unwrap())

                .build().unwrap(),

            Broker::OpenBroker => OldCommissionSpecBuilder::new()
                .minimum(Cash::new("RUB", dec!(0.04)))
                .percent(dec!(0.057))
                .build().unwrap(),
        }
    }
}

impl<'de> Deserialize<'de> for Broker {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = String::deserialize(deserializer)?;

        Ok(match value.as_str() {
            "bcs" => Broker::Bcs,
            "interactive-brokers" => Broker::InteractiveBrokers,
            "open-broker" => Broker::OpenBroker,

            _ => return Err(D::Error::unknown_variant(&value, &[
                "bcs", "interactive-brokers", "open-broker",
            ])),
        })
    }
}

#[derive(Debug, Clone)]
pub struct BrokerInfo {
    pub name: &'static str,
    config: BrokerConfig,
    commission_spec: OldCommissionSpec,
    pub allow_sparse_broker_statements: bool,
}

impl BrokerInfo {
    pub fn get_deposit_commission(&self, assets: CashAssets) -> GenericResult<Decimal> {
        let currency = assets.cash.currency;

        let commission_spec = match self.config.deposit_commissions.get(currency) {
            Some(commission_spec) => commission_spec,
            None => return Err!(concat!(
                "Unable to calculate commission for {} deposit to {}: there is no commission ",
                "specification in the configuration file"), currency, self.name),
        };

        Ok(commission_spec.fixed_amount)
    }

    pub fn get_trade_commission(&self, trade_type: TradeType, shares: u32, price: Cash) -> GenericResult<Cash> {
        self.commission_spec.calculate(trade_type, shares, price)
    }
}