
import {
  getBinanceAccounts,
  makeBinanceRequestParams, makeBinanaceOrigRequests,
  makeBinanceSpotRequestParams, makeBinanaceSpotOrigRequests
} from "./utils";
import { ZkTLSClient, ProverClient, saveToFile } from "@primuslabs/por-client-sdk";


function getBinanaceRequestParams() {
  const accounts = getBinanceAccounts();
  // console.log('accounts', accounts);
  const origRequests = makeBinanaceOrigRequests(accounts);
  // console.log('origRequests', origRequests);
  const { requests, responseResolves } = makeBinanceRequestParams(origRequests);
  // console.log('requests', requests);
  // console.log('responseResolves', responseResolves);
  return { requests, responseResolves };
}

function getBinanaceSpotRequestParams() {
  const accounts = getBinanceAccounts();
  // console.log('accounts', accounts);
  const origRequests = makeBinanaceSpotOrigRequests(accounts);
  // console.log('origRequests', origRequests);
  const { requests, responseResolves } = makeBinanceSpotRequestParams(origRequests);
  // console.log('requests', requests);
  // console.log('responseResolves', responseResolves);
  return { requests, responseResolves };
}

async function main() {
  console.log(`Now: ${new Date()}`);
  // Configure at least one or more pairs of API_KEY and API_SECRET
  // in .env using `BINANCE_API_KEY{i}` and `BINANCE_API_SECRET{i}`.
  try {
    let data1: any = null;
    let data2: any = null;
    const zktlsClient = new ZkTLSClient();
    {
      const { requests, responseResolves } = getBinanaceRequestParams();
      data1 = await zktlsClient.doZkTLS(requests, responseResolves, {
        requestParamsCallback: getBinanaceRequestParams,
      });
      if (data1 && data1.attestationData) {
        saveToFile("attestation1.json", JSON.stringify(data1.attestationData));
      }
    }
    {
      const { requests, responseResolves } = getBinanaceSpotRequestParams();
      data2 = await zktlsClient.doZkTLS(requests, responseResolves, {
        requestParamsCallback: getBinanaceSpotRequestParams,
      });
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
