/**
 * In-memory indexer for pool analytics. Production deployments should
 * back this with Horizon/Soroban RPC event ingestion.
 */

export type PoolEventType =
  | "swap"
  | "add_liquidity"
  | "remove_liquidity"
  | "campaign_created"
  | "reward_distributed";

export interface PoolEvent {
  id: string;
  poolId: string;
  type: PoolEventType;
  timestamp: number;
  payload: Record<string, string | number>;
}

export interface PoolStats {
  poolId: string;
  tokenA: string;
  tokenB: string;
  tvl: number;
  volume24h: number;
  fees24h: number;
  swapCount: number;
}

export interface Position {
  id: string;
  poolId: string;
  owner: string;
  shares: number;
  valueUsd: number;
}

export interface PricePoint {
  poolId: string;
  timestamp: number;
  price: number;
  feeBps: number;
}

export class PoolIndexer {
  private stats = new Map<string, PoolStats>();
  private events: PoolEvent[] = [];
  private positions = new Map<string, Position>();
  private priceHistory: PricePoint[] = [];

  indexEvent(event: PoolEvent): void {
    this.events.push(event);
    const stats = this.stats.get(event.poolId) ?? {
      poolId: event.poolId,
      tokenA: String(event.payload.tokenA ?? ""),
      tokenB: String(event.payload.tokenB ?? ""),
      tvl: 0,
      volume24h: 0,
      fees24h: 0,
      swapCount: 0,
    };

    if (event.type === "swap") {
      stats.swapCount += 1;
      stats.volume24h += Number(event.payload.amountIn ?? 0);
      stats.fees24h += Number(event.payload.fee ?? 0);
    }
    if (event.type === "add_liquidity") {
      stats.tvl += Number(event.payload.amountA ?? 0) + Number(event.payload.amountB ?? 0);
    }
    this.stats.set(event.poolId, stats);
  }

  getPoolStats(poolId?: string): PoolStats[] {
    const all = [...this.stats.values()];
    return poolId ? all.filter((s) => s.poolId === poolId) : all;
  }

  getEvents(poolId?: string, limit = 100): PoolEvent[] {
    const filtered = poolId
      ? this.events.filter((e) => e.poolId === poolId)
      : this.events;
    return filtered.slice(-limit).reverse();
  }

  getPositions(owner?: string): Position[] {
    const all = [...this.positions.values()];
    return owner ? all.filter((p) => p.owner === owner) : all;
  }

  upsertPosition(position: Position): void {
    this.positions.set(position.id, position);
  }

  recordPrice(point: PricePoint): void {
    this.priceHistory.push(point);
  }

  getPriceHistory(poolId: string, from?: number, to?: number): PricePoint[] {
    return this.priceHistory.filter((p) => {
      if (p.poolId !== poolId) return false;
      if (from && p.timestamp < from) return false;
      if (to && p.timestamp > to) return false;
      return true;
    });
  }
}

export const defaultIndexer = new PoolIndexer();

// Seed demo data for local development
defaultIndexer.indexEvent({
  id: "evt-1",
  poolId: "pool-demo",
  type: "swap",
  timestamp: Date.now() - 60_000,
  payload: { amountIn: 1000, fee: 3, tokenA: "XLM", tokenB: "USDC" },
});
defaultIndexer.upsertPosition({
  id: "pos-1",
  poolId: "pool-demo",
  owner: "G...DEMO",
  shares: 5000,
  valueUsd: 12_500,
});
defaultIndexer.recordPrice({
  poolId: "pool-demo",
  timestamp: Date.now(),
  price: 0.12,
  feeBps: 30,
});
