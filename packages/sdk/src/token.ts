/**
 * TokenClient — typed client for the SEP-41 LP token contract.
 *
 * Covers the public interface of contracts/token/src/lib.rs.
 */

import {
  Contract,
  rpc as StellarRpc,
  nativeToScVal,
  scValToNative,
  xdr,
  Address,
} from "@stellar/stellar-sdk";
import type { NetworkConfig } from "./types.js";

// ── Helpers ────────────────────────────────────────────────────────────────────

function addr(address: string): xdr.ScVal {
  return nativeToScVal(Address.fromString(address));
}

function i128(value: bigint): xdr.ScVal {
  return nativeToScVal(value, { type: "i128" });
}

// ── TokenClient ───────────────────────────────────────────────────────────────

export class TokenClient {
  private readonly server: StellarRpc.Server;
  private readonly contract: Contract;
  private readonly networkPassphrase: string;

  constructor(config: NetworkConfig) {
    this.server = new StellarRpc.Server(config.rpcUrl);
    this.contract = new Contract(config.contractId);
    this.networkPassphrase = config.networkPassphrase;
  }

  get contractId(): string {
    return this.contract.contractId();
  }

  private async simulate(method: string, ...args: xdr.ScVal[]): Promise<xdr.ScVal> {
    const op = this.contract.call(method, ...args);
    const tx = new (await import("@stellar/stellar-sdk")).TransactionBuilder(
      await this.server.getAccount("GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN"),
      { fee: "100", networkPassphrase: this.networkPassphrase }
    )
      .addOperation(op)
      .setTimeout(30)
      .build();
    const result = await this.server.simulateTransaction(tx);
    if (StellarRpc.Api.isSimulationError(result)) {
      throw new Error(result.error);
    }
    return (result as StellarRpc.Api.SimulateTransactionSuccessResponse).result!.retval;
  }

  // ── Read-only methods ──────────────────────────────────────────────────────

  /** Returns the token name. */
  async name(): Promise<string> {
    const raw = await this.simulate("name");
    return String(scValToNative(raw));
  }

  /** Returns the token symbol. */
  async symbol(): Promise<string> {
    const raw = await this.simulate("symbol");
    return String(scValToNative(raw));
  }

  /** Returns the number of decimal places used to represent token amounts. */
  async decimals(): Promise<number> {
    const raw = await this.simulate("decimals");
    return Number(scValToNative(raw));
  }

  /** Returns the total number of tokens currently in circulation. */
  async totalSupply(): Promise<bigint> {
    const raw = await this.simulate("total_supply");
    return BigInt(String(scValToNative(raw)));
  }

  /** Returns the token balance of `address`. Returns `0n` if no balance. */
  async balance(address: string): Promise<bigint> {
    const raw = await this.simulate("balance", addr(address));
    return BigInt(String(scValToNative(raw)));
  }

  /**
   * Returns the amount `spender` is allowed to transfer on behalf of `from`.
   * Returns `0n` if no allowance has been set.
   */
  async allowance(from: string, spender: string): Promise<bigint> {
    const raw = await this.simulate("allowance", addr(from), addr(spender));
    return BigInt(String(scValToNative(raw)));
  }

  // ── Write-method parameter types ───────────────────────────────────────────
  //
  // These methods require a signed transaction envelope. The parameter types
  // are provided here to support typed integration layers; submitting the
  // transaction is the caller's responsibility using the Stellar SDK.

  /** Parameters for `transfer(from, to, amount)`. */
  transferParams(from: string, to: string, amount: bigint): xdr.ScVal[] {
    return [addr(from), addr(to), i128(amount)];
  }

  /** Parameters for `transfer_from(spender, from, to, amount)`. */
  transferFromParams(spender: string, from: string, to: string, amount: bigint): xdr.ScVal[] {
    return [addr(spender), addr(from), addr(to), i128(amount)];
  }

  /** Parameters for `approve(from, spender, amount)`. */
  approveParams(from: string, spender: string, amount: bigint): xdr.ScVal[] {
    return [addr(from), addr(spender), i128(amount)];
  }

  /** Parameters for `mint(to, amount)` — admin only. */
  mintParams(to: string, amount: bigint): xdr.ScVal[] {
    return [addr(to), i128(amount)];
  }

  /** Parameters for `burn(from, amount)` — admin only. */
  burnParams(from: string, amount: bigint): xdr.ScVal[] {
    return [addr(from), i128(amount)];
  }
}
