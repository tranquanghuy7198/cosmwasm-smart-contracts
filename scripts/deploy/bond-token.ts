import dotenv from "dotenv";
import fs from "fs";
import { baseDeploy, compile, ContractInfo } from '../base-deploy';
import contractAddresses from "../../contract-addresses.json";

dotenv.config();

const NETWORK = "malaga";
const ADMIN = "wasm10w2pwzxaacsj508ma5ruz5wnhn83tld7mvxkuk";

const BOND_TOKEN = "bond_token";

let deploy = async () => {
  let contracts: ContractInfo[] = [
    {
      contractName: BOND_TOKEN,
      initParams: {
        issuer: "wasm10w2pwzxaacsj508ma5ruz5wnhn83tld7mvxkuk",
        basic_info: {
          name: "Bond Token",
          symbol: "BOND-TOKEN", // Note: By default, the symbol must satisfy the regex [a-zA-Z\-]{3-12}
          decimals: 18,
          initial_balances: [{ address: ADMIN, amount: "1000000" }],
          mint: { minter: ADMIN, cap: "1234567890" }
        },
        function_setup: {
          transfer: true,
          burn: true,
          mint_to_investor: true,
          subscribe: true,
        },
        additional_data: "Lorem ipsum",
        currency: "wasm10w2pwzxaacsj508ma5ruz5wnhn83tld7mvxkuk",
        placeholder: "wasm10w2pwzxaacsj508ma5ruz5wnhn83tld7mvxkuk",
        router: "wasm10w2pwzxaacsj508ma5ruz5wnhn83tld7mvxkuk",
        denomination: {
          currency_amount: "4",
          bond_amount: "7"
        }
      }
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