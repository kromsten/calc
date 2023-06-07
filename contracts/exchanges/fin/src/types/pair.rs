use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use exchange::pair::Pair as ExchangePair;

use super::position_type::PositionType;

#[cw_serde]
pub struct Pair {
    pub base_denom: String,
    pub quote_denom: String,
    pub address: Addr,
    pub decimal_delta: i8,
    pub price_precision: u8,
}

impl Pair {
    pub fn position_type(&self, swap_denom: &str) -> PositionType {
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

impl Into<ExchangePair> for Pair {
    fn into(self) -> ExchangePair {
        ExchangePair {
            denoms: [self.base_denom, self.quote_denom],
        }
    }
}
