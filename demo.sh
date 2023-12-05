# xCONST - sARCH (ratio)

# 1_000_000 -> 262_649        Price: 0.262649
# Reverse:
# 1_000_000 -> 3_807_363      Price: 3.807363

# Pair Direct:
echo "Directly quering estimates from the pair contract:"
archwayd q wasm contract-state smart archway1903dqer5mdy4wen9duxhm7l76gw20vzk2vwm6t7zk305c0m38ldqjncc9f '{  "swap_simulation" : { "swap_from_asset_index" : 0, "swap_to_asset_index" : 1, "amount" : "1000000"  }  }'
echo "Reverse"
archwayd q wasm contract-state smart archway1903dqer5mdy4wen9duxhm7l76gw20vzk2vwm6t7zk305c0m38ldqjncc9f '{  "swap_simulation" : { "swap_from_asset_index" : 1, "swap_to_asset_index" : 0, "amount" : "1000000"  }  }'

# Exchange:
echo "\nExpected amounts from the exchange contract:"
archwayd q wasm contract-state smart archway1lhwam0n77ysx7478pw8505ljckqv9ka4lllwxdqaqx573trw6hnssrkgxc '{ "get_expected_receive_amount": { "swap_amount": { "denom":  "archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka", "amount" : "1000000" }, "target_denom": "archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp"   }  }'
archwayd q wasm contract-state smart archway1lhwam0n77ysx7478pw8505ljckqv9ka4lllwxdqaqx573trw6hnssrkgxc '{ "get_expected_receive_amount": { "swap_amount": { "denom":  "archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp", "amount" : "1000000" }, "target_denom": "archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka"   }  }'

# Twap:
echo "\nTwap xConst -> sARCH:"
archwayd q wasm contract-state smart archway1lhwam0n77ysx7478pw8505ljckqv9ka4lllwxdqaqx573trw6hnssrkgxc '{ "get_twap_to_now": { "target_denom": "archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp", "swap_denom": "archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka", "period" : 0   }  }'
echo "Twap sARCH -> xConst:"
archwayd q wasm contract-state smart archway1lhwam0n77ysx7478pw8505ljckqv9ka4lllwxdqaqx573trw6hnssrkgxc '{ "get_twap_to_now": { "target_denom": "archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka", "swap_denom": "archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp", "period" : 0   }  }'

# Testing swaps:
echo "\nBalances before swap:"
echo "xCONST:"
archwayd q wasm contract-state smart archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka '{ "balance": { "address": "archway1dvpkaw4wmcn05k7v6c98cv4g9mgdamukghcwk2"  }  }'
echo "sARCH:"
archwayd q wasm contract-state smart archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp '{ "balance": { "address": "archway1dvpkaw4wmcn05k7v6c98cv4g9mgdamukghcwk2"  }  }'
echo "\nSwapping 1_000_000_000 xCONST for sARCH:"

archwayd tx wasm execute archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka  '{ "send" : { "contract": "archway1lhwam0n77ysx7478pw8505ljckqv9ka4lllwxdqaqx573trw6hnssrkgxc", "amount": "1000000000", "msg": "ewogICJzd2FwIjogewogICAgIm1pbmltdW1fcmVjZWl2ZV9hbW91bnQiOiB7CiAgICAgICJhbW91bnQiOiAiMTAwMDAwMCIsCiAgICAgICJkZW5vbSI6ICJhcmNod2F5MWVycWd1cWMzaG1mYWpndTdlMmR2Z2FjY3g2ZmV1NXJ1M2d5YXRkeHU5NHA2Nmo5aHA3bXNuMmtjcXAiCiAgICB9CiAgfQp9"  }  }' --gas-prices 900000000000aconst --from test  --gas 1600000 -y

echo "\Sleeping for 5 seconds..."
sleep 5

echo "\nBalances after swap:"
echo "xCONST:"
archwayd q wasm contract-state smart archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka '{ "balance": { "address": "archway1dvpkaw4wmcn05k7v6c98cv4g9mgdamukghcwk2"  }  }'
echo "sARCH:"
archwayd q wasm contract-state smart archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp '{ "balance": { "address": "archway1dvpkaw4wmcn05k7v6c98cv4g9mgdamukghcwk2"  }  }'


# archwayd tx wasm execute archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka  '{ "send" : { "contract": "archway1lhwam0n77ysx7478pw8505ljckqv9ka4lllwxdqaqx573trw6hnssrkgxc", "amount": "501000000", "msg": "ewogICJzd2FwIjogewogICAgIm1pbmltdW1fcmVjZWl2ZV9hbW91bnQiOiB7CiAgICAgICJhbW91bnQiOiAiMTAwMDAwMCIsCiAgICAgICJkZW5vbSI6ICJhY29uc3QiCiAgICB9CiAgfQp9"  }  }' --gas-prices 900000000000aconst --from test  --gas 5000000 -y
# archwayd tx wasm execute archway1lhwam0n77ysx7478pw8505ljckqv9ka4lllwxdqaqx573trw6hnssrkgxc '{ "swap": { "minimum_receive_amount": { "amount": "1000000000", "denom": "archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka"  } }  }' --gas-prices 900000000000aconst --from test -y --amount 1000000000aconst --gas 1600000

# archwayd tx wasm execute archway1lhwam0n77ysx7478pw8505ljckqv9ka4lllwxdqaqx573trw6hnssrkgxc '{ "swap": { "minimum_receive_amount": { "amount": "1000000", "denom": "archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka"  } }  }' --gas-prices 900000000000aconst --from test -y --amount 1500000000000000000aconst --gas 1600000