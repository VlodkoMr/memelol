# LOL MemeCoin
Meme coin on NEAR

### Launch on localhost
1. ```yarn dev```
2. відкрити в браузері https://near.org/flags і вписати порт http://127.0.0.1:4040/ тоді "Save Flags"
3. перейти на https://near.org/memelol.near/widget/lol.App


### Build Smart-contracts

```
npm run build:contracts
```

Run Testnet
=============

### Deploy new smart-contracts:

```
./dev-deploy-clear.sh
```

### Update smart-contracts:

```
npm run dev:contract:update
```

## Call smart-contract:
```
NEAR_ID=vlodkow5.testnet
CONTRACT_ID=$(<neardev/dev-account)
LP_CONTRACT_ID=liquidity.$CONTRACT_ID
BURN_CONTRACT_ID=burn.$CONTRACT_ID
```

##### Open Box
``` 
near call $CONTRACT_ID open_box '' --accountId $NEAR_ID --deposit 0.05
```

##### Get user rewards
```
near view $CONTRACT_ID get_user_rewards '{"owner_id":"'$NEAR_ID'"}'
```

##### Get total stats
```
near view $CONTRACT_ID get_total_stats
```

##### Get leaderboards
```
near view $CONTRACT_ID get_leaderboards ''
```

##### Get total supply
```
near view $CONTRACT_ID ft_total_supply ''
```

##### Get user token balance
```
near view $CONTRACT_ID ft_balance_of '{"account_id":"'$NEAR_ID'"}'
```

##### Transfer LOL tokens
```
near call $CONTRACT_ID ft_transfer '{"receiver_id":"'$LP_CONTRACT_ID'","amount":"1000"}' --accountId $NEAR_ID --depositYocto 1
```

##### Get balances on contracts
```
near view $CONTRACT_ID ft_balance_of '{"account_id":"'$CONTRACT_ID'"}'
near view $CONTRACT_ID ft_balance_of '{"account_id":"'$LP_CONTRACT_ID'"}'
near view $CONTRACT_ID ft_balance_of '{"account_id":"'$BURN_CONTRACT_ID'"}'
```

##### Admin method: add premium boxes for user
```
ACCOUNT_ID=vlodkow5.testnet
ADD_COUNT=10
near call $CONTRACT_ID add_additional_premium '{"account_id":"'$ACCOUNT_ID'","amount":'$ADD_COUNT'}' --accountId $NEAR_ID
```

##### Admin method: get count of premium boxes left for user
```
ACCOUNT_ID=vlodkow5.testnet
near view $CONTRACT_ID user_premium_boxes_left '{"account_id":"'$ACCOUNT_ID'"}'
```
