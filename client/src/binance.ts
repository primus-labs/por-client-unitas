import { DataSource, ZkTLSClient, ProverClient, saveToFile } from "@primuslabs/por-client-sdk";

async function main() {
  console.log(`Now: ${new Date()}`);
  try {
    const ds = new DataSource.Binance();
    let data1: any = null;
    let data2: any = null;
    const zktlsClient = new ZkTLSClient();
    {
      const requestParams = ds.getUnifiedAccountRequests();
      data1 = await zktlsClient.doZkTLS(requestParams);
      if (data1 && data1.attestationData) {
        saveToFile("attestation1.json", JSON.stringify(data1.attestationData));
      }
    }
    {
      const requestParams = ds.getSpotAccountRequests();
      data2 = await zktlsClient.doZkTLS(requestParams);
      if (data2 && data2.attestationData) {
        saveToFile("attestation2.json", JSON.stringify(data2.attestationData));
      }
    }
    {
      const zkVmRequestData = {
        "unified": data1.attestationData,
        "spot": data2.attestationData,
      };
      const proverClient = new ProverClient();
      const submitResult = await proverClient.submitTask(JSON.stringify(zkVmRequestData))
      // console.log("submitResult", submitResult);
      const result = await proverClient.getResult(submitResult.task_id);
      // console.log("result", result);
      console.log('proof fixture(json):', JSON.parse(result?.details?.proof_fixture ?? "{}"));
    }
  } catch (error) {
    console.log('main error:', error);
  }
}

const interval = Number(process.env.INTERVAL) || 1800;
console.log(`The interval: ${interval} s.`)
main();
setInterval(main, interval * 1000);
