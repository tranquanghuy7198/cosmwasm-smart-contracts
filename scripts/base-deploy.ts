import dotenv from "dotenv";
import { execSync } from 'child_process';
import fs from "fs";
import { calculateFee } from '@cosmjs/stargate';
import { JsonObject } from '@cosmjs/cosmwasm-stargate';
import { NETWORKS } from '../config';
import { initialize } from './base';
import promptSync from "prompt-sync";

dotenv.config();

const prompter = promptSync();
const MNEMONIC = process.env.MNEMONIC;

export type ContractInfo = {
  contractName: string,
  initParams: JsonObject;
};

export type DeployResult = {
  codeId: number,
  address: String;
};

export const compile = async () => {
  await askPermission("Do you want to compile the smart contracts?", async () => {
    console.log("Compiling...");
    let optimizeResult = execSync(
      `sudo docker run --rm -v "$(pwd)":/code \\
      --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \\
      --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \\
      cosmwasm/workspace-optimizer-arm64:0.12.11`
    ).toString();
  });
};

export const baseDeploy = async (info: ContractInfo, network: string): Promise<DeployResult> => {
  let deployResult: DeployResult = { codeId: 0, address: "" };
  await askPermission(`Do you want to upload ${info.contractName.toUpperCase()} wasm bytecode to blockchain?`, async () => {

    // Validate network
    let networkConfig = NETWORKS.find(n => n.name === network);
    if (!networkConfig) {
      console.error(`Unknown network: ${network}`);
      return;
    }

    // Prepare deployer account
    let [account, client] = await initialize(networkConfig).setup(MNEMONIC);
    let balance = await client.getBalance(account, networkConfig.feeToken);
    console.log(`Deploying ${info.contractName}...`);
    console.log(`Deployer: ${account}`);
    console.log(`Balance: ${balance.amount}${balance.denom}`);

    let wasmPath = `artifacts/${info.contractName}-aarch64.wasm`;
    if (!fs.existsSync(wasmPath)) {
      console.error(`Unknown contract: ${info.contractName}`);
      return { codeId: 0, address: "" };
    }

    // Upload wasm binary
    let wasm = fs.readFileSync(wasmPath);
    let uploadFee = calculateFee(networkConfig.fees.upload, networkConfig.gasPrice);
    let uploadResult = await client.upload(account, wasm, uploadFee);
    deployResult.codeId = uploadResult.codeId;
    console.log(`${info.contractName} code ID: ${uploadResult.codeId}`);

    // Instantiate contract
    await askPermission(`Do you want to instantiate ${info.contractName.toUpperCase()} contract using this code ID?`, async () => {
      let instantiateResponse = await client.instantiate(
        account,
        uploadResult.codeId,
        info.initParams,
        info.contractName,
        calculateFee(networkConfig.fees.init, networkConfig.gasPrice)
      );
      deployResult.address = instantiateResponse.contractAddress;
      console.log(`${info.contractName}: ${instantiateResponse.contractAddress}`);
    });
  });

  return deployResult;
};

let askPermission = async (question: string, action: Function) => {
  let answer = "";
  do {
    answer = prompter(question + " (y/N) ");
  } while (!["y", "Y", "n", "N"].includes(answer));
  if (answer === "y" || answer === "N")
    await action();
};
