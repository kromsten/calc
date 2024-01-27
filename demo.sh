# The script expects the following:

# 1. archwayd configured to a correct testnet node
# 2. archwayd has a key named "test" with gas funds
# 3. "test" address already has some xCONST

CONTRACT=archway14akhhwmuwzs8mfh4vwtctx23gh8naxmkajmwpfqx5se7k4x3tmrq2vh37u



# xCONST - sARCH (ratio)
# 1_000_000 -> 262_649        Price: 0.262649
# Reverse:
# 1_000_000 -> 3_807_363      Price: 3.807363

# Pair Direct:
echo "Directly quering estimates from the pair contract:"
XARCHBEF=$( archwayd q wasm contract-state smart archway1903dqer5mdy4wen9duxhm7l76gw20vzk2vwm6t7zk305c0m38ldqjncc9f '{  "swap_simulation" : { "swap_from_asset_index" : 0, "swap_to_asset_index" : 1, "amount" : "1000000"  }  }' | jq  .data.to_amount_minus_fee -r )
XCONSTBEF=$( archwayd q wasm contract-state smart archway1903dqer5mdy4wen9duxhm7l76gw20vzk2vwm6t7zk305c0m38ldqjncc9f '{  "swap_simulation" : { "swap_from_asset_index" : 1, "swap_to_asset_index" : 0, "amount" : "1000000"  }  }' | jq  .data.to_amount_minus_fee -r )

# Exchange:
echo "\nExpected amounts from the exchange contract:"
XARCHAFT=$(  archwayd q wasm contract-state smart $CONTRACT '{ "get_expected_receive_amount": { "swap_amount": { "denom":  "archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka", "amount" : "1000000" }, "target_denom": "archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp"   }  }'  | jq  .data.amount  -r )
XCONSTAFT=$(  archwayd q wasm contract-state smart $CONTRACT '{ "get_expected_receive_amount": { "swap_amount": { "denom":  "archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp", "amount" : "1000000" }, "target_denom": "archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka"   }  }' | jq  .data.amount  -r )

echo "$XARCHBEF == $XARCHAFT"
echo "$XCONSTBEF == $XCONSTAFT"


# Twap:
echo "\nTwap xConst -> sARCH:"
archwayd q wasm contract-state smart $CONTRACT '{ "get_twap_to_now": { "target_denom": "archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp", "swap_denom": "archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka", "period" : 0   }  }'
echo "Twap sARCH -> xConst:"
archwayd q wasm contract-state smart $CONTRACT '{ "get_twap_to_now": { "target_denom": "archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka", "swap_denom": "archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp", "period" : 0   }  }'


# Testing swaps:
echo "\nBalances before swap:"
echo "xCONST:"
archwayd q wasm contract-state smart archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka '{ "balance": { "address": "archway1dvpkaw4wmcn05k7v6c98cv4g9mgdamukghcwk2"  }  }'
echo "sARCH:"
archwayd q wasm contract-state smart archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp '{ "balance": { "address": "archway1dvpkaw4wmcn05k7v6c98cv4g9mgdamukghcwk2"  }  }'
echo "USDC.axv:"
archwayd q wasm contract-state smart archway1mmu32f7hn0fyw8gh57xl5uhaqu4pq5xx59ynf0tju60n2nzha0as3vtmcm '{ "balance": { "address": "archway1dvpkaw4wmcn05k7v6c98cv4g9mgdamukghcwk2"  }  }'
echo "USDT.axv:"
archwayd q wasm contract-state smart archway12defza8nz2d2a3hgt6tftku28ly5lglsna69j7rpjeukg4pz8qese2326s '{ "balance": { "address": "archway1dvpkaw4wmcn05k7v6c98cv4g9mgdamukghcwk2"  }  }'
echo "BUSD.axv:"
archwayd q wasm contract-state smart archway1lcxaem4gqnp2mdeh5hawacxlzge8e6wzjaat73ahpf2rclqv9v4sj8e4kf '{ "balance": { "address": "archway1dvpkaw4wmcn05k7v6c98cv4g9mgdamukghcwk2"  }  }'



# Ratio Swap:

echo "\nSwapping 10_000_000_000_000 xCONST for sARCH:"
archwayd tx wasm execute archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka  '{ "send" : { "contract": "", "amount": "10000000000000", "msg": "ewogICJzd2FwIjogewogICAgIm1pbmltdW1fcmVjZWl2ZV9hbW91bnQiOiB7CiAgICAgICJhbW91bnQiOiAiMTAwMDAwMCIsCiAgICAgICJkZW5vbSI6ICJhcmNod2F5MWVycWd1cWMzaG1mYWpndTdlMmR2Z2FjY3g2ZmV1NXJ1M2d5YXRkeHU5NHA2Nmo5aHA3bXNuMmtjcXAiCiAgICB9CiAgfQp9"  }  }' --gas-prices 900000000000aconst --from test  --gas 1600000 -y
echo "\Sleeping for 5 seconds..."
sleep 5


# Standard Swap:
echo "\nSwapping 10_000_000_000_000 xCONST for USDC.axv:"
archwayd tx wasm execute archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka  '{ "send" : { "contract": "archway14akhhwmuwzs8mfh4vwtctx23gh8naxmkajmwpfqx5se7k4x3tmrq2vh37u", "amount": "10000000000000", "msg": "ewogICJzd2FwIjogewogICAgIm1pbmltdW1fcmVjZWl2ZV9hbW91bnQiOiB7CiAgICAgICJhbW91bnQiOiAiMTAwMDAwMCIsCiAgICAgICJkZW5vbSI6ICJhcmNod2F5MW1tdTMyZjdobjBmeXc4Z2g1N3hsNXVoYXF1NHBxNXh4NTl5bmYwdGp1NjBuMm56aGEwYXMzdnRtY20iCiAgICB9CiAgfQp9"  }  }' --gas-prices 900000000000aconst --from test  --gas 1600000 -y

echo "\nSleeping for 5 seconds..."
sleep 5

echo "xCONST:"
archwayd q wasm contract-state smart archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka '{ "balance": { "address": "archway1dvpkaw4wmcn05k7v6c98cv4g9mgdamukghcwk2"  }  }'

echo "USDC.axv:"
archwayd q wasm contract-state smart archway1mmu32f7hn0fyw8gh57xl5uhaqu4pq5xx59ynf0tju60n2nzha0as3vtmcm '{ "balance": { "address": "archway1dvpkaw4wmcn05k7v6c98cv4g9mgdamukghcwk2"  }  }'

echo "USDT.axv:"
archwayd q wasm contract-state smart archway12defza8nz2d2a3hgt6tftku28ly5lglsna69j7rpjeukg4pz8qese2326s '{ "balance": { "address": "archway1dvpkaw4wmcn05k7v6c98cv4g9mgdamukghcwk2"  }  }'


# Stable Swaps:
echo "\nSwapping 1_000_000_000 (u)USDC.axv for USDT.axv"
archwayd tx wasm execute archway1mmu32f7hn0fyw8gh57xl5uhaqu4pq5xx59ynf0tju60n2nzha0as3vtmcm  '{ "send" : { "contract": "archway14akhhwmuwzs8mfh4vwtctx23gh8naxmkajmwpfqx5se7k4x3tmrq2vh37u", "amount": "1000000000", "msg": "ewogICJzd2FwIjogewogICAgIm1pbmltdW1fcmVjZWl2ZV9hbW91bnQiOiB7CiAgICAgICJhbW91bnQiOiAiOTAwIiwKICAgICAgImRlbm9tIjogImFyY2h3YXkxMmRlZnphOG56MmQyYTNoZ3Q2dGZ0a3UyOGx5NWxnbHNuYTY5ajdycGpldWtnNHB6OHFlc2UyMzI2cyIKICAgIH0KICB9Cn0="  }  }' --gas-prices 900000000000aconst --from test  --gas 1600000 -y

echo "\nSleeping for 3 seconds..."
sleep 3

echo "\nSwapping 1_000_000_000 (u)USDC.axv for BUSD.axv"
archwayd tx wasm execute archway1mmu32f7hn0fyw8gh57xl5uhaqu4pq5xx59ynf0tju60n2nzha0as3vtmcm  '{ "send" : { "contract": "archway14akhhwmuwzs8mfh4vwtctx23gh8naxmkajmwpfqx5se7k4x3tmrq2vh37u", "amount": "1000000000", "msg": "ewogICJzd2FwIjogewogICAgIm1pbmltdW1fcmVjZWl2ZV9hbW91bnQiOiB7CiAgICAgICJhbW91bnQiOiAiOTAwIiwKICAgICAgImRlbm9tIjogImFyY2h3YXkxbGN4YWVtNGdxbnAybWRlaDVoYXdhY3hsemdlOGU2d3pqYWF0NzNhaHBmMnJjbHF2OXY0c2o4ZTRrZiIKICAgIH0KICB9Cn0="  }  }' --gas-prices 900000000000aconst --from test  --gas 1600000 -y

echo "\nSleeping for 5 seconds..."
sleep 5


echo "\nFinal balances:"
echo "xCONST:"
archwayd q wasm contract-state smart archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka '{ "balance": { "address": "archway1dvpkaw4wmcn05k7v6c98cv4g9mgdamukghcwk2"  }  }'
echo "sARCH:"
archwayd q wasm contract-state smart archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp '{ "balance": { "address": "archway1dvpkaw4wmcn05k7v6c98cv4g9mgdamukghcwk2"  }  }'

echo "USDC.axv:"
archwayd q wasm contract-state smart archway1mmu32f7hn0fyw8gh57xl5uhaqu4pq5xx59ynf0tju60n2nzha0as3vtmcm '{ "balance": { "address": "archway1dvpkaw4wmcn05k7v6c98cv4g9mgdamukghcwk2"  }  }'

echo "USDT.axv:"
archwayd q wasm contract-state smart archway12defza8nz2d2a3hgt6tftku28ly5lglsna69j7rpjeukg4pz8qese2326s '{ "balance": { "address": "archway1dvpkaw4wmcn05k7v6c98cv4g9mgdamukghcwk2"  }  }'

echo "BUSD.axv:"
archwayd q wasm contract-state smart archway1lcxaem4gqnp2mdeh5hawacxlzge8e6wzjaat73ahpf2rclqv9v4sj8e4kf '{ "balance": { "address": "archway1dvpkaw4wmcn05k7v6c98cv4g9mgdamukghcwk2"  }  }'



# archwayd tx wasm execute $CONTRACT  '{ "send" : { "contract": "archway14akhhwmuwzs8mfh4vwtctx23gh8naxmkajmwpfqx5se7k4x3tmrq2vh37u", "amount": "501000000", "msg": "ewogICJzd2FwIjogewogICAgIm1pbmltdW1fcmVjZWl2ZV9hbW91bnQiOiB7CiAgICAgICJhbW91bnQiOiAiMTAwMDAwMCIsCiAgICAgICJkZW5vbSI6ICJhY29uc3QiCiAgICB9CiAgfQp9"  }  }' --gas-prices 900000000000aconst --from test  --gas 5000000 -y
# archwayd tx wasm execute $CONTRACT '{ "swap": { "minimum_receive_amount": { "amount": "1000000000", "denom": "archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka"  } }  }' --gas-prices 900000000000aconst --from test -y --amount 1000000000aconst --gas 1600000

# archwayd tx wasm execute $CONTRACT '{ "swap": { "minimum_receive_amount": { "amount": "1000000", "denom": "archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka"  } }  }' --gas-prices 900000000000aconst --from test -y --amount 1500000000000000000aconst --gas 1600000