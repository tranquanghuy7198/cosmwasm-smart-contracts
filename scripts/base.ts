import axios from "axios";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { makeCosmoshubPath } from "@cosmjs/proto-signing";
import { NetworkConfig } from '../config';
import { Secp256k1HdWallet } from '@cosmjs/amino';

interface Network {
  setup: (mnemonic: string | undefined) => Promise<[string, SigningCosmWasmClient]>;
}

export const initialize = (config: NetworkConfig): Network => {

  /* Create a HD wallet with 20 first accounts generated from the given mnemonic */
  const createWallet = async (mnemonic: string): Promise<Secp256k1HdWallet> => {
    let hdPaths = [...Array(20).keys()].map(i => makeCosmoshubPath(i));
    return await Secp256k1HdWallet.fromMnemonic(mnemonic, { prefix: config.bech32prefix, hdPaths });
  };

  const connect = async (
    wallet: Secp256k1HdWallet,
    config: NetworkConfig
  ): Promise<SigningCosmWasmClient> => {
    const clientOptions = { prefix: config.bech32prefix };
    return await SigningCosmWasmClient.connectWithSigner(config.rpc, wallet, clientOptions);
  };

  const hitFaucet = async (
    faucetUrl: string,
    address: string,
    denom: string
  ): Promise<void> => {
    await axios.post(faucetUrl, { denom, address });
  };

  const setup = async (mnemonic: string | undefined): Promise<[string, SigningCosmWasmClient]> => {
    if (!mnemonic)
      throw new Error("Mnemonic is undefined");

    const wallet = await createWallet(mnemonic);
    const client = await connect(wallet, config);

    const [account] = await wallet.getAccounts();
    // ensure we have some tokens
    if (config.faucetUrl) {
      const tokens = await client.getBalance(account.address, config.feeToken);
      if (tokens.amount === '0') {
        console.log(`Getting ${config.feeToken} from faucet...`);
        await hitFaucet(config.faucetUrl, account.address, config.feeToken);
      }
    }

    return [account.address, client];
  };

  return { setup };
};