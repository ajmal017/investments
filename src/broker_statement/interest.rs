use crate::core::GenericResult;
use crate::currency::Cash;
use crate::currency::converter::CurrencyConverter;
use crate::localities::Country;
use crate::types::{Date, Decimal};

#[derive(Debug)]
pub struct IdleCashInterest {
    pub date: Date,
    pub amount: Cash,
}

impl IdleCashInterest {
    pub fn tax_to_pay(&self, country: &Country, converter: &CurrencyConverter) -> GenericResult<Decimal> {
        let amount = converter.convert_to_rounding(self.date, self.amount, country.currency)?;
        Ok(country.tax_to_pay(amount, None))
    }
}