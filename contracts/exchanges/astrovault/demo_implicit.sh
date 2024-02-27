# The script expects the following:

# 1. archwayd configured to a correct testnet node
# 2. archwayd has a key named "test" with gas funds
# 3. "test" address already has some xCONST

CONTRACT=archway1y9k5al4r4hn5vrshsvrmq0cepcptpvk6s7hwy0a4w73dl4qjhfvqq6nwgh
USER=archway1dvpkaw4wmcn05k7v6c98cv4g9mgdamukghcwk2


#  Pair map:
# 
#  The demo expect the paits from pairs.json to be created using "internal_msg" from prior
#  On top of existing pairs from pairs.json intermediary ones are also created
#
#  Get full pair info
#  archwayd q wasm contract-state smart $CONTRACT '{  "internal_query" : { "msg": "ewogICJnZXRfcGFpcnNfZnVsbCI6IHt9Cn0="  }     }'  | jq

#  Explciit Pairs:

#  aCONST  
#               ->  sARCH  archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp 

#  xCONST archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka
#               ->  BUSD.axv  archway1lcxaem4gqnp2mdeh5hawacxlzge8e6wzjaat73ahpf2rclqv9v4sj8e4kf

#
#  Implciit Pool Pairs:
#
#  aCONST
#    stable     -> xCONST archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka

#  xCONST archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka
#    ratio      -> sARCH    archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp
#    standard   -> USDC.axv archway1mmu32f7hn0fyw8gh57xl5uhaqu4pq5xx59ynf0tju60n2nzha0as3vtmcm 
#
#  USDC.axv archway1mmu32f7hn0fyw8gh57xl5uhaqu4pq5xx59ynf0tju60n2nzha0as3vtmcm
#    stable     -> USDT.axv archway12defza8nz2d2a3hgt6tftku28ly5lglsna69j7rpjeukg4pz8qese2326s
#  
#  USDT.axv archway12defza8nz2d2a3hgt6tftku28ly5lglsna69j7rpjeukg4pz8qese2326s
#    stable     -> BUSD.axv archway1lcxaem4gqnp2mdeh5hawacxlzge8e6wzjaat73ahpf2rclqv9v4sj8e4kf 
#
#  Implciit Routes:
#
#  xCONST archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka 
#        -> USDT.axv archway12defza8nz2d2a3hgt6tftku28ly5lglsna69j7rpjeukg4pz8qese2326s  
#        
#  USDC.axv archway1mmu32f7hn0fyw8gh57xl5uhaqu4pq5xx59ynf0tju60n2nzha0as3vtmcm 
#        -> BUSD.axv  archway1lcxaem4gqnp2mdeh5hawacxlzge8e6wzjaat73ahpf2rclqv9v4sj8e4kf




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
archwayd q wasm contract-state smart archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka '{ "balance": { "address": "'$USER'"  }  }'
echo "sARCH:"
archwayd q wasm contract-state smart archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp '{ "balance": { "address": "'$USER'"  }  }'
echo "USDC.axv:"
archwayd q wasm contract-state smart archway1mmu32f7hn0fyw8gh57xl5uhaqu4pq5xx59ynf0tju60n2nzha0as3vtmcm '{ "balance": { "address": "'$USER'"  }  }'
echo "USDT.axv:"
archwayd q wasm contract-state smart archway12defza8nz2d2a3hgt6tftku28ly5lglsna69j7rpjeukg4pz8qese2326s '{ "balance": { "address": "'$USER'"  }  }'
echo "BUSD.axv:"
archwayd q wasm contract-state smart archway1lcxaem4gqnp2mdeh5hawacxlzge8e6wzjaat73ahpf2rclqv9v4sj8e4kf '{ "balance": { "address": "'$USER'"  }  }'



# Ratio Swap:
echo "\nSwapping 10_000_000_000_000 xCONST for sARCH  "
archwayd tx wasm execute archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka  '{ "send" : { "contract": "'$CONTRACT'", "amount": "10000000000000", "msg": "ewogICJzd2FwIjogewogICAgIm1pbmltdW1fcmVjZWl2ZV9hbW91bnQiOiB7CiAgICAgICJhbW91bnQiOiAiMTAwMDAwMCIsCiAgICAgICJkZW5vbSI6ICJhcmNod2F5MWVycWd1cWMzaG1mYWpndTdlMmR2Z2FjY3g2ZmV1NXJ1M2d5YXRkeHU5NHA2Nmo5aHA3bXNuMmtjcXAiCiAgICB9CiAgfQp9"  }  }' --gas-prices 900000000000aconst --from test  --gas 2000000 -y
echo "\Sleeping for 5 seconds..."
sleep 5


# Standard Swap:
echo "\nSwapping 10_000_000_000_000 xCONST for USDC.axv:"
archwayd tx wasm execute archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka  '{ "send" : { "contract": "'$CONTRACT'", "amount": "10000000000000", "msg": "ewogICJzd2FwIjogewogICAgIm1pbmltdW1fcmVjZWl2ZV9hbW91bnQiOiB7CiAgICAgICJhbW91bnQiOiAiMTAwMDAwMCIsCiAgICAgICJkZW5vbSI6ICJhcmNod2F5MW1tdTMyZjdobjBmeXc4Z2g1N3hsNXVoYXF1NHBxNXh4NTl5bmYwdGp1NjBuMm56aGEwYXMzdnRtY20iCiAgICB9CiAgfQp9"  }  }' --gas-prices 900000000000aconst --from test  --gas 2000000 -y

echo "\nSleeping for 5 seconds..."
sleep 5
echo "xCONST:"
archwayd q wasm contract-state smart archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka '{ "balance": { "address": "'$USER'"  }  }'
echo "USDC.axv:"
archwayd q wasm contract-state smart archway1mmu32f7hn0fyw8gh57xl5uhaqu4pq5xx59ynf0tju60n2nzha0as3vtmcm '{ "balance": { "address": "'$USER'"  }  }'
echo "USDT.axv:"
archwayd q wasm contract-state smart archway12defza8nz2d2a3hgt6tftku28ly5lglsna69j7rpjeukg4pz8qese2326s '{ "balance": { "address": "'$USER'"  }  }'


# Stable Swaps:
echo "\nSwapping 1_000_000_000 (u)USDC.axv for USDT.axv"
archwayd tx wasm execute archway1mmu32f7hn0fyw8gh57xl5uhaqu4pq5xx59ynf0tju60n2nzha0as3vtmcm  '{ "send" : { "contract": "'$CONTRACT'", "amount": "1000000000", "msg": "ewogICJzd2FwIjogewogICAgIm1pbmltdW1fcmVjZWl2ZV9hbW91bnQiOiB7CiAgICAgICJhbW91bnQiOiAiOTAwIiwKICAgICAgImRlbm9tIjogImFyY2h3YXkxMmRlZnphOG56MmQyYTNoZ3Q2dGZ0a3UyOGx5NWxnbHNuYTY5ajdycGpldWtnNHB6OHFlc2UyMzI2cyIKICAgIH0KICB9Cn0="  }  }' --gas-prices 900000000000aconst --from test  --gas 2400000 -y
echo "\nSleeping for 6 seconds..."
sleep 6
echo "\nSwapping 1_000_000_000 (u)USDC.axv for BUSD.axv"
archwayd tx wasm execute archway1mmu32f7hn0fyw8gh57xl5uhaqu4pq5xx59ynf0tju60n2nzha0as3vtmcm  '{ "send" : { "contract": "'$CONTRACT'", "amount": "1000000000", "msg": "ewogICJzd2FwIjogewogICAgIm1pbmltdW1fcmVjZWl2ZV9hbW91bnQiOiB7CiAgICAgICJhbW91bnQiOiAiOTAwIiwKICAgICAgImRlbm9tIjogImFyY2h3YXkxbGN4YWVtNGdxbnAybWRlaDVoYXdhY3hsemdlOGU2d3pqYWF0NzNhaHBmMnJjbHF2OXY0c2o4ZTRrZiIKICAgIH0KICB9Cn0="  }  }' --gas-prices 900000000000aconst --from test  --gas 3500000 -y
echo "\nSleeping for 6 seconds..."
sleep 6


echo "\nRouted Swap: aCONST -> (xCONST) -> sARCH \n"

echo "\nBalances before swap:"
aCONST_B=$( archwayd q bank balances "$USER" | jq -r '.balances.[0].amount')
sARCH_B=$( archwayd q wasm contract-state smart archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp '{ "balance": { "address": "'$USER'"  }  }' | jq -r .data.balance )
echo "aCONST: $aCONST_B"
echo "sARCH: $sARCH_B"


echo "\nEstimating amounts manually from each hop pool"
ACONST_TO_XCONST=$(  archwayd q wasm contract-state smart $CONTRACT '{ "get_expected_receive_amount": { "swap_amount": { "denom":  "aconst", "amount" : "1000000" }, "target_denom": "archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka"   }  }'  | jq  .data.amount  -r )
XCONST_TO_SARCH=$(  archwayd q wasm contract-state smart $CONTRACT '{ "get_expected_receive_amount": { "swap_amount": { "denom":  "archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka", "amount" : "'$ACONST_TO_XCONST'" }, "target_denom": "archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp"   }  }'  | jq  .data.amount  -r )

echo "1_000_000 aCONST -> $ACONST_TO_XCONST xCONST"
echo "$ACONST_TO_XCONST xCONST -> $XCONST_TO_SARCH sARCH"

echo "\nEstimate directly from the contract:"
ACONST_TO_SARCH=$( archwayd q wasm contract-state smart $CONTRACT '{ "get_expected_receive_amount": { "swap_amount": { "denom":  "aconst", "amount" : "1000000" }, "target_denom": "archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp"   }  }'  | jq  .data.amount  -r )
echo "1_000_000 aCONST -> $ACONST_TO_SARCH sARCH"

echo "Difference between manual and direct estimation: $ACONST_TO_SARCH - $XCONST_TO_SARCH = $(($ACONST_TO_SARCH-$XCONST_TO_SARCH))"

echo "Swapping 1_000_000 aCONST for sARCH:"
archwayd tx wasm execute $CONTRACT '{ "swap": { "minimum_receive_amount": { "amount": "180000", "denom": "archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp"  } }  }' --gas-prices 900000000000aconst --from test -y --amount 1000000aconst --gas 3000000
echo "\nSleeping for 7 seconds..."
sleep 7

echo "\nBalances after swap:"
aCONST_A=$( archwayd q bank balances "$USER" | jq -r '.balances.[0].amount')
sARCH_A=$( archwayd q wasm contract-state smart archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp '{ "balance": { "address": "'$USER'"  }  }' | jq -r .data.balance )
echo "aCONST: $aCONST_A"
echo "sARCH: $sARCH_A"


# not running comparioson for aCONST due to gas overhead
echo "$sARCH_B sARCH + $ACONST_TO_SARCH sARCH should be ≈ $(($sARCH_B+$ACONST_TO_SARCH)) sARCH. We have $sARCH_A after the swap" 
echo "difference is  $(($sARCH_A - ($sARCH_B+$ACONST_TO_SARCH)))"


echo "\nRouted Swap: xCONST -> (USDC, USDT) -> BUSD.axv \n"

echo "\nBalances before swap:"
XCONST_B=$( archwayd q wasm contract-state smart archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka '{ "balance": { "address": "'$USER'"  }  }' | jq -r .data.balance ) 
BUSD_B=$( archwayd q wasm contract-state smart archway1lcxaem4gqnp2mdeh5hawacxlzge8e6wzjaat73ahpf2rclqv9v4sj8e4kf '{ "balance": { "address": "'$USER'"  }  }' | jq -r .data.balance )

echo "xCONST: $XCONST_B"
echo "BUSD.axv: $BUSD_B"

echo "\nEstimating amounts manually from each hop pool"

XCONST_TO_USDC=$(  archwayd q wasm contract-state smart $CONTRACT '{ "get_expected_receive_amount": { "swap_amount": { "denom":  "archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka", "amount" : "10000000" }, "target_denom": "archway1mmu32f7hn0fyw8gh57xl5uhaqu4pq5xx59ynf0tju60n2nzha0as3vtmcm"   }  }'  | jq  .data.amount  -r )
USDC_TO_USDT=$(  archwayd q wasm contract-state smart $CONTRACT '{ "get_expected_receive_amount": { "swap_amount": { "denom":  "archway1mmu32f7hn0fyw8gh57xl5uhaqu4pq5xx59ynf0tju60n2nzha0as3vtmcm", "amount" : "'$XCONST_TO_USDC'" }, "target_denom": "archway12defza8nz2d2a3hgt6tftku28ly5lglsna69j7rpjeukg4pz8qese2326s"   }  }'  | jq  .data.amount  -r )
USDT_TO_BUSD=$(  archwayd q wasm contract-state smart $CONTRACT '{ "get_expected_receive_amount": { "swap_amount": { "denom":  "archway12defza8nz2d2a3hgt6tftku28ly5lglsna69j7rpjeukg4pz8qese2326s", "amount" : "'$USDC_TO_USDT'" }, "target_denom": "archway1lcxaem4gqnp2mdeh5hawacxlzge8e6wzjaat73ahpf2rclqv9v4sj8e4kf"   }  }'  | jq  .data.amount  -r )

echo "10_000_000 xCONST -> $XCONST_TO_USDC USDC.axv"
echo "$XCONST_TO_USDC USDC.axv -> $USDC_TO_USDT USDT.axv"
echo "$USDC_TO_USDT USDT.axv -> $USDT_TO_BUSD BUSD.axv"


echo "\nEstimate directly from the contract:"
XCONST_TO_BUSD=$( archwayd q wasm contract-state smart $CONTRACT '{ "get_expected_receive_amount": { "swap_amount": { "denom":  "archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka", "amount" : "10000000" }, "target_denom": "archway1lcxaem4gqnp2mdeh5hawacxlzge8e6wzjaat73ahpf2rclqv9v4sj8e4kf"   }  }'  | jq  .data.amount  -r )
echo "10_000_000 xCONST -> $XCONST_TO_BUSD BUSD.axv"

echo "Difference between manual and direct estimation: $XCONST_TO_BUSD - $USDT_TO_BUSD = $(($XCONST_TO_BUSD-$USDT_TO_BUSD))"

echo "\nSwapping 10_000_000 xCONST for BUSD.axv:"
archwayd tx wasm execute archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka  '{ "send" : { "contract": "'$CONTRACT'", "amount": "10000000", "msg": "ewogICJzd2FwIjogewogICAgIm1pbmltdW1fcmVjZWl2ZV9hbW91bnQiOiB7CiAgICAgICJhbW91bnQiOiAiOSIsCiAgICAgICJkZW5vbSI6ICJhcmNod2F5MWxjeGFlbTRncW5wMm1kZWg1aGF3YWN4bHpnZThlNnd6amFhdDczYWhwZjJyY2xxdjl2NHNqOGU0a2YiCiAgICB9CiAgfQp9"  }  }' --gas-prices 900000000000aconst --from test  --gas 4300000 -y
echo "\nSleeping for 6 seconds..."
sleep 6

echo "\nBalances after swap:"
XCONST_A=$( archwayd q wasm contract-state smart archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka '{ "balance": { "address": "'$USER'"  }  }' | jq -r '.data.balance' ) 
USDC_A=$( archwayd q wasm contract-state smart archway1mmu32f7hn0fyw8gh57xl5uhaqu4pq5xx59ynf0tju60n2nzha0as3vtmcm '{ "balance": { "address": "'$USER'"  }  }' | jq -r '.data.balance' )
echo "xCONST: $XCONST_A"
echo "USDC.axv: $USDC_A"


echo "\nSleeping for 6 seconds..."
sleep 6




echo "\nFinal balances:"
echo "xCONST:"
archwayd q wasm contract-state smart archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka '{ "balance": { "address": "'$USER'"  }  }'
echo "sARCH:"
archwayd q wasm contract-state smart archway1erqguqc3hmfajgu7e2dvgaccx6feu5ru3gyatdxu94p66j9hp7msn2kcqp '{ "balance": { "address": "'$USER'"  }  }'
echo "USDC.axv:"
archwayd q wasm contract-state smart archway1mmu32f7hn0fyw8gh57xl5uhaqu4pq5xx59ynf0tju60n2nzha0as3vtmcm '{ "balance": { "address": "'$USER'"  }  }'
echo "USDT.axv:"
archwayd q wasm contract-state smart archway12defza8nz2d2a3hgt6tftku28ly5lglsna69j7rpjeukg4pz8qese2326s '{ "balance": { "address": "'$USER'"  }  }'
echo "BUSD.axv:"
archwayd q wasm contract-state smart archway1lcxaem4gqnp2mdeh5hawacxlzge8e6wzjaat73ahpf2rclqv9v4sj8e4kf '{ "balance": { "address": "'$USER'"  }  }'





# archwayd tx wasm execute $CONTRACT  '{ "send" : { "contract": "'$CONTRACT'", "amount": "501000000", "msg": "ewogICJzd2FwIjogewogICAgIm1pbmltdW1fcmVjZWl2ZV9hbW91bnQiOiB7CiAgICAgICJhbW91bnQiOiAiMTAwMDAwMCIsCiAgICAgICJkZW5vbSI6ICJhY29uc3QiCiAgICB9CiAgfQp9"  }  }' --gas-prices 900000000000aconst --from test  --gas 5000000 -y
# archwayd tx wasm execute $CONTRACT '{ "swap": { "minimum_receive_amount": { "amount": "1000000000", "denom": "archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka"  } }  }' --gas-prices 900000000000aconst --from test -y --amount 1000000000aconst --gas 1600000
# archwayd tx wasm execute $CONTRACT '{ "swap": { "minimum_receive_amount": { "amount": "1000000", "denom": "archway1sdzaas0068n42xk8ndm6959gpu6n09tajmeuq7vak8t9qt5jrp6sjjtnka"  } }  }' --gas-prices 900000000000aconst --from test -y --amount 1500000000000000000aconst --gas 1600000