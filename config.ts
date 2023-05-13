import { GasPrice } from "@cosmjs/stargate";

export interface NetworkConfig {
  readonly name: string;
  readonly rpc: string;
  readonly networkId: string;
  readonly feeToken: string;
  readonly bech32prefix: string;
  readonly faucetUrl?: string;
  readonly gasPrice: GasPrice;
  readonly fees: {
    upload: number,
    init: number,
    exec: number;
  };
}

export const NETWORKS: NetworkConfig[] = [
  {
    name: "malaga",
    rpc: "https://rpc.malaga-420.cosmwasm.com",
    networkId: "malaga-420",
    bech32prefix: "wasm",
    feeToken: "umlg",
    faucetUrl: "https://faucet.malaga-420.cosmwasm.com/credit",
    gasPrice: GasPrice.fromString("0.05umlg"),
    fees: {
      upload: 6000000,
      init: 1000000,
      exec: 500000,
    },
  },
  {
    name: "juno",
    rpc: "https://rpc.uni.kingnodes.com/status",
    networkId: "uni-5",
    bech32prefix: "juno",
    feeToken: "ujunox",
    faucetUrl: "https://faucet.uni.juno.deuslabs.fi/credit",
    gasPrice: GasPrice.fromString("0.025ujunox"),
    fees: {
      upload: 6000000,
      init: 500000,
      exec: 200000,
    },
  }
];
