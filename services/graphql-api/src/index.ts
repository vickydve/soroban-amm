import { ApolloServer } from "@apollo/server";
import { startStandaloneServer } from "@apollo/server/standalone";
import { PubSub } from "graphql-subscriptions";
import { typeDefs } from "./schema.js";
import { defaultIndexer } from "./indexer.js";

const pubsub = new PubSub();

const resolvers = {
  Query: {
    poolStats: (_: unknown, { poolId }: { poolId?: string }) =>
      defaultIndexer.getPoolStats(poolId),
    poolEvents: (
      _: unknown,
      { poolId, limit }: { poolId?: string; limit?: number },
    ) => defaultIndexer.getEvents(poolId, limit ?? 100),
    positions: (_: unknown, { owner }: { owner?: string }) =>
      defaultIndexer.getPositions(owner),
    priceHistory: (
      _: unknown,
      { poolId, from, to }: { poolId: string; from?: number; to?: number },
    ) => defaultIndexer.getPriceHistory(poolId, from, to),
    twal: () => null, // populated by indexer integration with twal_consumer contract
  },
  PoolEvent: {
    payload: (parent: { payload: Record<string, unknown> }) =>
      JSON.stringify(parent.payload),
  },
  Subscription: {
    poolEvent: {
      subscribe: (_: unknown, { poolId }: { poolId?: string }) =>
        pubsub.asyncIterator(poolId ? `EVENT:${poolId}` : "EVENT:ALL"),
    },
  },
};

export async function startServer(port = 4000) {
  const server = new ApolloServer({ typeDefs, resolvers });
  const { url } = await startStandaloneServer(server, {
    listen: { port },
  });
  return url;
}

startServer().then((url) => console.log(`GraphQL API ready at ${url}`));
