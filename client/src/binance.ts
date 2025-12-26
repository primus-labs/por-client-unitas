import { DataSource, PoRClient } from "@primuslabs/por-client-sdk";

async function main() {
  console.log(`Now: ${new Date()}`);
  try {
    const ds = new DataSource.Binance();
    const requestParams1 = ds.getUnifiedAccountRequests();
    const requestParams2 = ds.getSpotAccountRequests();

    const client = new PoRClient();
    const result = await client.run([requestParams1, requestParams2], {
      requestParamsCallback: [
        () => ds.getUnifiedAccountRequests(),
        () => ds.getSpotAccountRequests(),
      ]
    });
    console.log('proof fixture(json):', JSON.parse(result?.proof_fixture ?? "{}"));
  } catch (err: any) {
    console.log("err:", err);
  }
}

const interval = Number(process.env.INTERVAL) || 1800;
console.log(`The interval: ${interval} s.`)
main();
setInterval(main, interval * 1000);
