# Near Smart Contracts

## Prerequisites

4. Rust
   - `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
   - `source $HOME/.cargo/env`
   - `rustup target add wasm32-unknown-unknown`
5. Near-cli (https://docs.near.org/docs/tools/near-cli): `npm install -g near-cli`

## Getting Started

### Testing

1. Smart contract testing
   - `npm run test-contract`

### Smart Contract development/deployment

1. Build smart contract
   - `npm run build-contracts`
2. Connect to NEAR blockchain
   - `near login`
   - it will open a browser tab where you will have to authorize near cli to make transactions on behalf of your account. After authorization keys will be stored in `~/.near-credentials`
   - Alternatively, if you have keys you can place those in `~/.near-credentials/default` for near-cli to use
3. Deploy smart contract
   - `npm run deploy-contracts`

## Interacting with smart contracts

- near call `ACCOUNT_WHERE_CONTRACT_IS_DEPLOYED` `METHOD_NAME` `ARGS` --accountId `ACCOUNT_THAT_SIGNED_TRANSACTION`
- `near delete marketplace.momentize.testnet momentize.testnet`. To delete master account
- `near create-account marketplace.momentize.testnet --masterAccount momentize.testnet`. To create child account


## NFT smart contract methods
- `near call nft.momentize.testnet new '{"owner_id":"nft.momentize.testnet", "metadata":{"spec":"momentize-nft-1.0.0","name":"Momentize NFT","symbol":"MOMENTNFT"}, "supply_cap_by_type": {"unique":"1", "subscription":"100000", "reward":"100000"} }' --accountId nft.momentize.testnet`. Call this method to initialize NFT contract. The contract has been initialized.

- `near call nft.momentize.testnet nft_approve '{"token_id":"2","account_id":"marketplace.momentize.testnet", "msg":"{ \"sale_conditions\": [{\"ft_or_st_token_id\":\"st.momentize.testnet\",\"price\":\"2\",\"st_symbol\":\"zee\"}] }"}' --accountId zeeshan.testnet --amount 1`. Call this method to approve NFT to some other account, mainly to `marketplace.momentize.testnet` to list NFT for sale. This method will trigger cross-contract call.

- `near call nft.momentize.testnet nft_revoke '{"token_id":"1","account_id":"nft.momentize.testnet"}' --accountId zeeshan.testnet --amount "00000000000000001"`. Call this method to remove access of given account.

- `near call nft.momentize.testnet nft_token '{"token_id":"1"}' --accountId nft.momentize.testnet`. Call this method to get the metadata of NFT token.

- `near call nft.momentize.testnet nft_total_supply --accountId nft.momentize.testnet`. Call this method to get the total NFTs minted by the given contract.

- `near call nft.momentize.testnet nft_mint '{"metadata":{"title":"zee X"}}' --accountId zeeshan.testnet --amount 21`. Call this method to mint NFT.

- `near call nft.momentize.testnet nft_transfer '{"token_id":"2", "receiver_id":"lucidspring.testnet"}' --accountId zeeshan.testnet --amount "0.000000000000000000000001"`. Call this method to transfer NFT. 

- `near call nft.momentize.testnet unlock_token_types '{"token_types":["unique"]}' --accountId nft.momentize.testnet` 

- `near call nft.momentize.testnet add_token_types '{"supply_cap_by_type":{"content":"100000"}}' --accountId nft.momentize.testnet` 

- `near call nft.momentize.testnet get_supply_caps --accountId nft.momentize.testnet` 

- `near call nft.momentize.testnet nft_tokens_for_type '{"token_type":"reward", "from_index":"0","limit":"100"}' --accountId nft.momentize.testnet`

- `near call nft.momentize.testnet nft_tokens_for_owner_by_type '{"account_id": "zeeshan.testnet", "token_type":"reward", "from_index":"0","limit":"100"}' --accountId nft.momentize.testnet`
## Marketplace smart contract methods
- `near call marketplace.momentize.testnet new '{"owner_id":"marketplace.momentize.testnet"}' --accountId marketplace.momentize.testnet`. Call this method to initialize marketplace contract. The contract has been initialized.

- `near call marketplace.momentize.testnet supported_ft_token_ids --accountId nft.momentize.testnet`.  Call this method to get the list of all currencies that are accepted in marketplace.

- `near call marketplace.momentize.testnet add_ft_or_st_token_ids '{"ft_or_st_token_id":"st.momentize.testnet"}' --accountId marketplace.momentize.testnet`. Call this method to add FT or ST to the list of acceptable currencies in marketplace.

- `near call marketplace.momentize.testnet storage_deposit '{"account_id":"zeeshan.testnet"}' --accountId marketplace.momentize.testnet --amount 1`.  Call this method to register account with the marketplace contract. The user will have to pay for the storage that will be used to list the NFT.

- `near call marketplace.momentize.testnet storage_paid '{"account_id":"zeeshan.testnet"}' --accountId marketplace.momentize.testnet`. Call this method to get the amount of storage paid by given account.

- `near call marketplace.momentize.testnet get_sales_by_nft_token_type '{"token_type":"unique", "from_index":"0", "limit":"10"}' --accountId marketplace.momentize.testnet`. Call this method to get all the tokens for sale filtered by `token_type`.

- `near call marketplace.momentize.testnet remove_sale '{"nft_contract_id":"nft.momentize.testnet", "token_id":"2"}' --accountId zeeshan.testnet --amount ".000000000000000000000001"`. Call this method to remove sale from marketplace.

- `near call marketplace.momentize.testnet offer '{"nft_contract_id":"nft.momentize.testnet", "token_id":"2", "ft_token_id":"ft.momentize.testnet", "bid_amount":"1"}' --accountId lucidspring.testnet --amount ".000000000000000000000001" --gas "300000000000000"`. Call this method to bid on some NFT. Bid amount should be greater than the last bid. If `bid_amount` is less than the asked price, the current bid will replace the last bid, if bid is equal to asked price it will trigger the purchase.


## Fungible token smart contract methods
- `near call ft.momentize.testnet new '{"owner_id":"ft.momentize.testnet","total_supply":"100000000","name":"momentize","symbol":"MOMENT","decimals":8}' --accountId ft.momentize.testnet` Call this method to initialize fungible token contract. The contract has been initialized.

- `near call ft.momentize.testnet ft_balance_of '{"account_id":"ft.momentize.testnet"}' --accountId ft.momentize.testnet`. Call this method to get balance of given account.

- `near call ft.momentize.testnet ft_total_supply '{"account_id":"ft.momentize.tesnet"}' --accountId ft.momentize.testnet`. Call this method to get total supply of FT.

- `near call ft.momentize.testnet storage_balance_of '{"account_id":"zeeshan.testnet"}'  --accountId ft.momentize.testnet`

- `near call ft.momentize.testnet storage_minimum_balance  --accountId ft.momentize.testnet`. Call this method to know minimum balance required to register the account.

- `near call ft.momentize.testnet storage_deposit '{"account_id":"zeeshan.testnet"}'  --accountId ft.momentize.testnet --amount ".00125"`.  Call this method to register account with the fungible token contract. This is only one time call.

- `near call ft.momentize.testnet storage_withdraw '{"amount":"1250000000000000000000"}'  --accountId zeeshan.testnet --amount ".000000000000000000000001"`. This method will deregister the account with fungible token contract.
- `near call ft.momentize.testnet ft_transfer '{"receiver_id":"zeeshan.testnet", "amount":"600"}' --accountId ft.momentize.testnet --amount ".000000000000000000000001"`. Call this method to transfer some FT. 

- `near call ft.momentize.testnet ft_transfer_from '{"sender_id":"zeeshan.testnet", "receiver_id":"noushan.testnet", "amount":"600"}' --accountId ft.momentize.testnet --amount ".000000000000000000000001"`. Call this method to transfer some FT. 

- `near call ft.momentize.testnet get_allowance '{"owner_id":"zeeshan.testnet", "escrow_account_id":"usecases.momentize.testnet"}' --accountId ft.momentize.testnet `. 

## Social token smart contract methods
- `near call st.momentize.testnet new '{"owner_id":"st.momentize.testnet", "name":"Momemtize ST"}' --accountId st.momentize.testnet`. Call this method to initialize social token contract. The contract has been initialized.

- `near call st.momentize.testnet get_social_token_supply '{"token_symbol":"zee"}' --accountId zeeshan.testnet`. Call this method to get supply of provided social token.

- `near call st.momentize.testnet get_social_token_owner '{"token_symbol":"zee"}' --accountId zeeshan.testnet`. Call this method to get the owner/creator of social token.

- `near call st.momentize.testnet get_balance '{"token_symbol":"zee", "account_id":"zeeshan.testnet"}' --accountId zeeshan.testnet`. Call this method to get ST balance provided `token_symbol` and `account_id`

- `near call st.momentize.testnet storage_deposit '{"token_symbol":"zee","account_id":"zeeshan.testnet"}' --accountId st.momentize.testnet --amount ".00277"`. Call this method to register account with the social token contract. This is only one time call.

- `near call st.momentize.testnet st_mint '{"token_symbol":"zee", "collateral_amount":"5"}' --accountId zeeshan.testnet`. Call this method to mint given social token using FT collateral.

- `near call st.momentize.testnet st_burn '{"token_symbol":"zee", "burn_amount":"5"}' --accountId zeeshan.testnet`. Call this method to burn given social tokens and release locked collateral.

## Oracle smart contract methods
- `near call oracle.momentize.testnet new '{"owner_id":"oracle.momentize.testnet"}' --accountId oracle.momentize.testnet`. Call this method to initialize Oracle contract.
- `near call oracle.momentize.testnet get_subscription_renewal_requests '{"max_num_accounts": "100", "max_requests": "100"}' --accountId oracle.momentize.testnet`. Call this method to get get_subscription_renewal_requests unfulfilled/pending requests.
- `near call oracle.momentize.testnet get_campaign_requests '{"max_num_accounts": "100", "max_requests": "100"}' --accountId oracle.momentize.testnet`. Call this method to get get_campaign_requests unfulfilled/pending requests.
- `near call oracle.momentize.testnet fulfill_request '{"account": "usecases.momentize.testnet", "nonce": "0", "data": "{\"user_id\":\"abc\",\"campaign_id\":\"1\",\"completed\":false}" }' --accountId oracle.momentize.testnet --gas 300000000000000 --amount ".000000000000000000000001"`
- `near call oracle.momentize.testnet add_authorization '{"node": "oracle-node.'$NEAR_ACCT'"}' --accountId oracle.momentize.testnet`
## Usecases smart contract methods
- `near call usecases.momentize.testnet new '{"owner_id": "usecases.momentize.testnet"}' --accountId usecases.momentize.testnet`
- `near call usecases.momentize.testnet add_oracle '{"oracle_account": "oracle.momentize.testnet"}' --accountId usecases.momentize.testnet  --gas 300000000000000`
- `near call usecases.momentize.testnet remove_oracle '{"oracle_account": "oracle.momentize.testnet"}' --accountId usecases.momentize.testnet  --gas 300000000000000`
- `near call usecases.momentize.testnet verify_campaign '{"user_id":"abc","oracle_account": "oracle.momentize.testnet", "campaign_id": 1}' --accountId nft.momentize.testnet  --gas 300000000000000 --amount ".00125"`
- `near call usecases.momentize.testnet get_received_vals '{"max": "100"}' --accountId usecases.momentize.testnet  --gas 300000000000000`
- `near call usecases.momentize.testnet get_received_val '{"nonce": "1"}' --accountId usecases.momentize.testnet  --gas 300000000000000`
- `near call usecases.momentize.testnet create_twitter_campaign '{"ft_account_id":"ft.momentize.testnet","ft_amount":"100","tweet_id": "17263", "max_claimants":"50", "operations": ["like","retweet"]}' --accountId zeeshan.testnet --gas 300000000000000 --amount 1`
- `near call usecases.momentize.testnet twitter_campaign '{"campaign_id":1}' --accountId usecases.momentize.testnet`
- `near call usecases.momentize.testnet twitter_campaigns '{"from_index": "0", "limit": "10"}' --accountId usecases.momentize.testnet  --gas 300000000000000`



- `near call usecases.momentize.testnet create_subscription_pass '{"content_reference":"https://abc.com","ft_account_id":"ft.momentize.testnet","pass_price":"35"}' --accountId zeeshan.testnet --gas 300000000000000 --amount 1`
- `near call usecases.momentize.testnet buy_subscription_pass '{"content_id":"53","max_renewals":"5"}' --accountId zeeshan.testnet --gas 300000000000000 --amount 1`
