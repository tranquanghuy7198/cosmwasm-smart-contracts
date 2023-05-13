import dotenv from "dotenv";
import fs from "fs";
import { baseDeploy, compile, ContractInfo } from '../base-deploy';
import contractAddresses from "../../contract-addresses.json";

dotenv.config();

const NETWORK = "malaga";

const CURRENCY = "currency";
const FACTORY = "factory";
const PLACEHOLDER = "placeholder";
const ROUTER = "router";

let deploy = async () => {
  let contracts: ContractInfo[] = [
    {
      contractName: CURRENCY,
      initParams: {
        name: "Tether USD",
        symbol: "USDT",
        decimals: 6,
        initial_balances: [{
          address: "wasm10w2pwzxaacsj508ma5ruz5wnhn83tld7mvxkuk",
          amount: "1234000000"
        }],
        mint: {
          minter: "wasm10w2pwzxaacsj508ma5ruz5wnhn83tld7mvxkuk",
          cap: "9999000000"
        },
      }
    },
    {
      contractName: FACTORY,
      initParams: {
        bond_token_code_id: "1234"
      }
    },
    {
      contractName: PLACEHOLDER,
      initParams: {}
    },
    {
      contractName: ROUTER,
      initParams: {}
    }
  ];

  await compile();

  for (const contract of contracts) {
    if (!contractAddresses[NETWORK])
      contractAddresses[NETWORK] = {} as any;
    contractAddresses[NETWORK][contract.contractName] = await baseDeploy(contract, NETWORK);
  }

  fs.writeFileSync("contract-addresses.json", JSON.stringify(contractAddresses, null, "\t"));
  console.log("Finish!");
};

deploy();