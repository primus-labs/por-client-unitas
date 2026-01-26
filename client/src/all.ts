import { Scheduler, DatasourceManager, PoRClient, loadConfigFromFile } from "@primuslabs/por-client-sdk";

const config = loadConfigFromFile();
let withdrawTime = Date.now();
const WITHDRAW_INTERVAL_MS = 3 * 24 * 60 * 60 * 1000; // 3 days
async function main() {
  const config = loadConfigFromFile();
  const client = new PoRClient(config.app);
  const ds = new DatasourceManager(config.datasource);

  try {
    const params = {
      binanceSpot: () => ds.binance?.getSpotAccountInfoRequests(),
      binanceUsdSFuture: () => ds.binance?.getUsdSFutureAccountBalanceV3Requests(),
      binanceUnified: () => ds.binance?.getUnifiedAccountBalanceRequests(),
      binanceMargin: () => ds.binance?.getMarginAccountBalanceRequests(),
      asterSpot: () => ds.aster?.getSpotAccountRequests(),
      asterUsdSFuture: () => ds.aster?.getUsdSFutureBalanceRequests(),
    };

    const result = await client.run(params);
    // console.log("result", JSON.stringify(result));
    console.log('proof fixture(json):', JSON.parse(result?.proof_fixture ?? "{}"));
  } catch (err: any) {
    console.log("main err:", err?.message, JSON.stringify(err));

    {
      const elapsedMs = Date.now() - withdrawTime;
      if (elapsedMs >= WITHDRAW_INTERVAL_MS) {
        const ret = await client.tryWithdrawBalance();
        if (ret) {
          withdrawTime = Date.now();
        }
      }
    }

    throw err;
  }
}

const scheduler = new Scheduler(main, {
  intervalMs: config.app.runtime.jobInterval * 1000, // ms
  stateFile: "/app/.state.json",
  shouldStop: (err) => {
    if (err?.data?.code === "timeout") return true;
    return false;
  }
});
scheduler.start();
