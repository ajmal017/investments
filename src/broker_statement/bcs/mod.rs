mod assets;
mod cash_flow;
mod common;
mod parser;
mod period;
mod trades;

use crate::brokers::{Broker, BrokerInfo};
use crate::config::Config;
use crate::core::GenericResult;
#[cfg(test)] use crate::taxes::TaxRemapping;

#[cfg(test)] use super::{BrokerStatement};
use super::{BrokerStatementReader, PartialBrokerStatement};

use parser::{Parser, SectionParser};

pub struct StatementReader {
    broker_info: BrokerInfo,
}

impl StatementReader {
    pub fn new(config: &Config) -> GenericResult<Box<dyn BrokerStatementReader>> {
        Ok(Box::new(StatementReader {
            broker_info: Broker::Bcs.get_info(config)?,
        }))
    }
}

impl BrokerStatementReader for StatementReader {
    fn is_statement(&self, path: &str) -> GenericResult<bool> {
        Ok(path.ends_with(".xls"))
    }

    fn read(&mut self, path: &str) -> GenericResult<PartialBrokerStatement> {
        Parser::read(self.broker_info.clone(), path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_real() {
        let statement = BrokerStatement::read(
            &Config::mock(), Broker::Bcs, "testdata/bcs", TaxRemapping::new(), true).unwrap();

        assert!(!statement.cash_flows.is_empty());
        assert!(!statement.cash_assets.is_empty());
        assert!(statement.idle_cash_interest.is_empty());

        assert!(!statement.stock_buys.is_empty());
        assert!(statement.stock_sells.is_empty());
        assert!(statement.dividends.is_empty());

        assert!(!statement.open_positions.is_empty());
        assert!(statement.instrument_names.is_empty());
    }
}