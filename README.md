#cosmwasm-smart-contract
New version of cosmwasm. that base on task
https://app.asana.com/0/1201450325862549/1203408298172853/f

## Compile and deploy

### Compile
checking image with your chip here
`https://github.com/CosmWasm/rust-optimizer`
Example below for intel chip on mac
```shell
$ sudo docker run --rm -v "$(pwd)":/code \
      --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
      --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
      cosmwasm/workspace-optimizer:0.12.11
```

We need to deploy bond token first to get its code ID, then use it as a parameter to deploy factory contract

### Deploy bond token

```shell
$ ts-node scripts/deploy/bond-token.ts
```

### Deploy other contracts

```shell
$ ts-node scripts/deploy/system.ts
```

## Deploy into AWS env Using Jenkins
- First, trigger deployment pipeline: https://jenkins.dev.interopera.co/job/pipeline-jobs/job/complie-smartcontract-cosmwasm/
      
- `release_version` just naming tag, for example `release_version` = `latest`
- Once pipeline success, exec into chainhub service
```
# get chainhub pod id
kubectl get pod -n interopera-apps-dev

# exec into chainhub pod
kubectl exec -it ${chainhub pod} bash -n interopera-apps-dev
```
- Verify complied binary is saved at: `/usr/src/app/smart-contract/cosmwasm/${release_version}`

## Next update

- Using route to mint bond token and currency token. create function mint bond and mint currency token:
   - MintToken (requestId, bondAddress, receivers, amounts, operatorAddress, signature) --> anyone can call this function. just validate signature
   - MintCurrency (requestId, currencyAddress, receivers, amounts, operatorAddress, signature) --> anyone can call this function, just validate signature

- Create new contract called asset_vault contracts using as a vault for receive, store, and send currency token/bond token (any token similar with CW-20)
   - constuctor: list operators
   - function: 
     - setOperators: list operator, list bool --> only admin can set operators
     - sendAsset(currencyAddress, requestIds, receivers, amounts) --> only operators can call


