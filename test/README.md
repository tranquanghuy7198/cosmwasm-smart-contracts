## Testing scenario

1. System admin set the fee percentage at 50%.

2. System admin creates a currency called `USDT` and mints to issuer and investors:
* `INVESTOR_1` has 1000 USDT.
* `INVESTOR_2` has 2000 USDT.
* `ISSUER` has 500 USDT.

3. `ISSUER` creates a bond token named `BOND-TOKEN` with the denomination at 3 USDT $\approx$ 2 BOND-TOKEN. This bond token is then registered to the placeholder.

4. `INVESTOR_1` subscribes 300 USDT for 200 BOND-TOKEN (he must send 600 USDT). `INVESTOR_2` subscribes 567 USDT for 378 BOND-TOKEN (he must send 1134 USDT). These amounts of USDT are then locked inside the placeholder.

5. `ISSUER` distributes bond token with the rule saying that:
* `INVESTOR_1` can subscribe at most 270 USDT.
* `INVESTOR_2` can subscribe at most 1000 USDT.

Due to this rule, 30 USDT is then returned to `INVESTOR_1` and 837 USDT is transferred to `ISSUER`.

6. `ISSUER` send `INVESTOR_1` 123 USDT and `INVESTOR_2` 321 USDT as coupons.

7. `ISSUER` call the bond token to estimate the redemption amount. The response is 837 USDT.

8. `ISSUER` pays 837 USDT to redeem the principals. `INVESTOR_1` receives 270 USDT back. `INVESTOR_2` receives 567 USDT back.

9. System admin withdraws all fees stored in the Placeholder contract.

## Changes of balances

|Step|`ISSUER`|>|`INVESTOR_1`|>|`INVESTOR_2`|
|:-:|:-:|:-:|:-:|:-:|:-:|
|^|`USDT`|`USDT`|`BOND-TOKEN`|`USDT`|`BOND-TOKEN`|
|2|500|1000|0|2000|0|
|4|500|400|0|866|0|
|5|1337|430|180|866|378|
|6|893|553|180|1187|378|
|8|56|823|0|1754|0|