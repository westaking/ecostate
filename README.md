# ecostate

escostate contract for phase-5 of regen-network testnet(kontraua)

### Upload contract
```
xrncli tx wasm store target/wasm32-unknown-unknown/release/cw_escostate.wasm --gas auto --from owner -y
```

### Instantiate contract
```
CODE_ID=<id>
INIT='{\"region\":\"region-1\",\"beneficiary\":\"$(xrncli keys show beneficiary -a)\",\"oracle\":\"$(xrncli keys show oracle -a)\",\"ecostate\":2500,\"total_tokens\":100000,\"payout_start_height\":478500,\"payout_end_height\":490000}'

xrncli tx wasm instantiate $CODE_ID "$INIT" --from owner --label "ecostate 1 <moniker>" -y
```

### Check my contract address
```
xrncli query wasm list-contract-by-code $CODE_ID

CONTRACT=<contract_address>
```

### Update ecostate
```
UPDATE_ECOSTATE='{"updateecostate":{"ecostate": 2710}}'
xrncli tx wasm execute $CONTRACT "$UPDATE_ECOSTATE" --from oracle -y
```

### Lock
```
CALL_LOCK='{"lock":{}}'
xrncli tx wasm execute $CONTRACT "$CALL_LOCK" --from owner -y  
```

### Unlock 
```
CALL_UNLOCK='{"unlock":{}}'
xrncli tx wasm execute $CONTRACT "$CALL_UNLOCK" --from owner -y 
```

### Change Beneficiary
```
CHANGE_BENEFICIARY="{\"changebeneficiary\":{\"beneficiary\": \"$(xrncli keys show testnet -a)\"}}"
xrncli tx wasm execute $CONTRACT "$CHANGE_BENEFICIARY" --from owner-y  
```

### Change Oracle
```
CHANGE_ORACLE="{\"changeoracle\":{\"oracle\": \"$(xrncli keys show testnet -a)\"}}"
xrncli tx wasm execute $CONTRACT "$CHANGE_ORACLE" --from owner -y  
```

### Change Owner
```
CHANGE_OWNER="{\"transferownership\":{\"owner\": \"$(xrncli keys show testnet -a)\"}}"
xrncli tx wasm execute $CONTRACT "$CHANGE_OWNER" --from owner -y  
```

### Query State
```
QUERY_STATE='{"state":{}}'
xrncli query wasm contract-state smart $CONTRACT "$QUERY_STATE"  -o json
```

### Query Balance
```
QUERY_BALANCE="{\"balance\":{\"address\": \"$(xrncli keys show beneficiary -a)\"}}"
xrncli query wasm contract-state smart $CONTRACT "$QUERY_BALANCE"  -o json
```







