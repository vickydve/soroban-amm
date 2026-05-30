export const typeDefs = `#graphql
  type PoolStats {
    poolId: ID!
    tokenA: String!
    tokenB: String!
    tvl: Float!
    volume24h: Float!
    fees24h: Float!
    swapCount: Int!
  }

  type PoolEvent {
    id: ID!
    poolId: ID!
    type: String!
    timestamp: Float!
    payload: String!
  }

  type Position {
    id: ID!
    poolId: ID!
    owner: String!
    shares: Float!
    valueUsd: Float!
  }

  type PricePoint {
    poolId: ID!
    timestamp: Float!
    price: Float!
    feeBps: Int!
  }

  type Query {
    poolStats(poolId: ID): [PoolStats!]!
    poolEvents(poolId: ID, limit: Int = 100): [PoolEvent!]!
    positions(owner: String): [Position!]!
    priceHistory(poolId: ID!, from: Float, to: Float): [PricePoint!]!
    twal(poolId: ID!, windowSeconds: Int!): Float
  }

  type Subscription {
    poolEvent(poolId: ID): PoolEvent!
  }
`;
