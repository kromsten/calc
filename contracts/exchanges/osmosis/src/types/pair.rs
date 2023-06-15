use cosmwasm_schema::cw_serde;
use exchange::pair::Pair as ExchangePair;

use super::position_type::PositionType;

#[cw_serde]
pub struct Pair {
    pub base_denom: String,
    pub quote_denom: String,
    pub route: Vec<u64>,
    pub decimal_delta: i8,
    pub price_precision: u8,
}

impl Pair {
    pub fn position_type(&self, swap_denom: String) -> PositionType {
        if self.quote_denom == swap_denom {
            PositionType::Enter
        } else {
            PositionType::Exit
        }
    }

    pub fn denoms(&self) -> [String; 2] {
        [self.base_denom.clone(), self.quote_denom.clone()]
    }

    pub fn other_denom(&self, swap_denom: String) -> String {
        if self.quote_denom == swap_denom {
            self.base_denom.clone()
        } else {
            self.quote_denom.clone()
        }
    }
}

impl From<Pair> for ExchangePair {
    fn from(val: Pair) -> Self {
        ExchangePair {
            denoms: [val.base_denom, val.quote_denom],
        }
    }
}
